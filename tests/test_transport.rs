use serde_json::json;
use visor::{
    AsyncVisorClient, ClientConfig, ListingInclude, ListingsFilter, VisorClient, VisorError,
};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn async_client(base_url: String) -> AsyncVisorClient {
    AsyncVisorClient::with_config(ClientConfig {
        api_key: "test-key".to_string(),
        base_url,
        ..ClientConfig::default()
    })
}

fn sync_client(base_url: String) -> VisorClient {
    VisorClient::with_config(ClientConfig {
        api_key: "test-key".to_string(),
        base_url,
        ..ClientConfig::default()
    })
}

fn listings_page_body() -> serde_json::Value {
    json!({
        "data": [{ "id": "abc123", "vin": "1HGCM82633A123456" }],
        "pagination": { "limit": 50, "offset": 0, "total": 1, "next_offset": null }
    })
}

// ── Auth header ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn async_auth_header_is_bearer_token() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page_body()))
        .expect(1)
        .mount(&server)
        .await;

    async_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .expect("request with correct auth header should succeed");
    // MockServer verifies expect(1) on drop
}

#[tokio::test]
async fn sync_auth_header_is_bearer_token() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page_body()))
        .expect(1)
        .mount(&server)
        .await;

    let server_uri = server.uri();
    tokio::task::spawn_blocking(move || {
        sync_client(server_uri)
            .filter_listings(&ListingsFilter::default())
            .expect("request with correct auth header should succeed");
    })
    .await
    .unwrap();
    // server drops here, verifying expect(1)
}

// ── Success decoding ──────────────────────────────────────────────────────────

#[tokio::test]
async fn async_200_decodes_listings_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page_body()))
        .mount(&server)
        .await;

    let page = async_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .expect("should decode successfully");

    assert_eq!(page.data.len(), 1);
    assert_eq!(page.data[0].id, "abc123");
    assert_eq!(page.data[0].vin, "1HGCM82633A123456");
    assert_eq!(page.pagination.total, 1);
}

#[tokio::test]
async fn sync_200_decodes_listings_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page_body()))
        .mount(&server)
        .await;

    let server_uri = server.uri();
    let page = tokio::task::spawn_blocking(move || {
        sync_client(server_uri)
            .filter_listings(&ListingsFilter::default())
            .expect("should decode successfully")
    })
    .await
    .unwrap();

    assert_eq!(page.data.len(), 1);
    assert_eq!(page.data[0].id, "abc123");
}

// ── Malformed success body → InvalidResponse (not TransportError) ─────────────

#[tokio::test]
async fn async_malformed_success_body_is_invalid_response_not_transport_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{ not valid json"))
        .mount(&server)
        .await;

    let err = async_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .unwrap_err();

    assert!(
        matches!(err, VisorError::InvalidResponse { .. }),
        "malformed JSON on 200 must be InvalidResponse, not TransportError; got: {err:?}"
    );
}

// ── Error dispatch ────────────────────────────────────────────────────────────

#[tokio::test]
async fn async_400_is_validation_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": { "code": "invalid_filter", "message": "bad param" }
        })))
        .mount(&server)
        .await;

    let err = async_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .unwrap_err();
    assert!(
        matches!(err, VisorError::ValidationError(_)),
        "got: {err:?}"
    );
}

#[tokio::test]
async fn async_401_is_auth_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": { "code": "unauthorized", "message": "bad key" }
        })))
        .mount(&server)
        .await;

    let err = async_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .unwrap_err();
    assert!(matches!(err, VisorError::AuthError(_)), "got: {err:?}");
}

#[tokio::test]
async fn async_429_preserves_retry_after_seconds() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "30")
                .set_body_json(json!({
                    "error": { "code": "rate_limited", "message": "slow down" }
                })),
        )
        .mount(&server)
        .await;

    let err = async_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .unwrap_err();

    match err {
        VisorError::RateLimitError { retry_after, .. } => {
            assert_eq!(retry_after, Some(std::time::Duration::from_secs(30)));
        }
        other => panic!("expected RateLimitError, got: {other:?}"),
    }
}

#[tokio::test]
async fn async_404_on_get_listing_is_not_found_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings/no-such-id"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": { "code": "not_found", "message": "listing not found" }
        })))
        .mount(&server)
        .await;

    let err = async_client(server.uri())
        .get_listing("no-such-id", None)
        .await
        .unwrap_err();
    assert!(matches!(err, VisorError::NotFoundError(_)), "got: {err:?}");
}

// ── Base URL composition ──────────────────────────────────────────────────────

#[tokio::test]
async fn async_trailing_slash_on_base_url_is_normalized() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page_body()))
        .expect(1)
        .mount(&server)
        .await;

    let base_url = format!("{}/", server.uri());
    AsyncVisorClient::with_config(ClientConfig {
        api_key: "test-key".to_string(),
        base_url,
        ..ClientConfig::default()
    })
    .filter_listings(&ListingsFilter::default())
    .await
    .expect("trailing slash should be normalized");
}

// ── get_listing: data envelope unwrap ────────────────────────────────────────

fn listing_detail_body() -> serde_json::Value {
    json!({
        "data": {
            "id": "abc123",
            "vin": "1HGCM82633A123456",
            "status": "active",
            "inventory_type": "used",
            "dealer": {
                "dealer_id": "d1",
                "name": "Test Dealer",
                "city": "San Francisco",
                "state": "CA"
            },
            "vehicle": {
                "vin": "1HGCM82633A123456",
                "status": "active",
                "build": { "year": 2020, "make": "Honda", "model": "Accord" }
            }
        },
        "meta": {}
    })
}

#[tokio::test]
async fn async_get_listing_decodes_data_envelope() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings/abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listing_detail_body()))
        .mount(&server)
        .await;

    let detail = async_client(server.uri())
        .get_listing("abc123", None)
        .await
        .expect("should decode { data: ListingDetail } envelope");

    assert_eq!(detail.id, "abc123");
    assert_eq!(detail.vin, "1HGCM82633A123456");
}

#[tokio::test]
async fn async_get_listing_sends_include_query_param() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings/abc123"))
        .and(query_param("include", "price_history,options"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listing_detail_body()))
        .expect(1)
        .mount(&server)
        .await;

    async_client(server.uri())
        .get_listing(
            "abc123",
            Some(vec![ListingInclude::PriceHistory, ListingInclude::Options]),
        )
        .await
        .expect("should send include query param");
    // MockServer verifies expect(1) on drop
}

#[tokio::test]
async fn sync_get_listing_decodes_data_envelope() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings/abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listing_detail_body()))
        .mount(&server)
        .await;

    let server_uri = server.uri();
    let detail = tokio::task::spawn_blocking(move || {
        sync_client(server_uri)
            .get_listing("abc123", None)
            .expect("should decode { data: ListingDetail } envelope")
    })
    .await
    .unwrap();

    assert_eq!(detail.id, "abc123");
    assert_eq!(detail.vin, "1HGCM82633A123456");
}

#[tokio::test]
async fn sync_get_listing_sends_include_query_param() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings/abc123"))
        .and(query_param("include", "price_history,options"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listing_detail_body()))
        .expect(1)
        .mount(&server)
        .await;

    let server_uri = server.uri();
    tokio::task::spawn_blocking(move || {
        sync_client(server_uri)
            .get_listing(
                "abc123",
                Some(vec![ListingInclude::PriceHistory, ListingInclude::Options]),
            )
            .expect("should send include query param");
    })
    .await
    .unwrap();
    // MockServer verifies expect(1) on drop
}

// ── Malformed error body → fallback to unknown_error ─────────────────────────

#[tokio::test]
async fn async_non_json_error_body_falls_back_to_unknown_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Service Unavailable"))
        .mount(&server)
        .await;

    let err = async_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .unwrap_err();

    match err {
        VisorError::VisorApiError(body) => {
            assert_eq!(body.code, "unknown_error");
            assert_eq!(body.message, "Service Unavailable");
        }
        other => panic!("expected VisorApiError, got: {other:?}"),
    }
}
