use serde_json::json;
use visor::{AsyncVisorClient, ClientConfig, ListingsFilter, VisorError};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_client(base_url: String) -> AsyncVisorClient {
    AsyncVisorClient::with_config(ClientConfig {
        api_key: "test-key".to_string(),
        base_url,
        ..ClientConfig::default()
    })
}

// ── Error dispatch (HTTP status → VisorError variant) ────────────────────────

#[cfg(feature = "phase-contracts")]
#[tokio::test]
async fn http_400_becomes_validation_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": { "code": "invalid_filter", "message": "Unknown field 'foo'" }
        })))
        .mount(&server)
        .await;

    let err = make_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .unwrap_err();
    assert!(
        matches!(err, VisorError::ValidationError(_)),
        "got: {err:?}"
    );
}

#[cfg(feature = "phase-contracts")]
#[tokio::test]
async fn http_401_becomes_auth_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": { "code": "unauthorized", "message": "Invalid API key" }
        })))
        .mount(&server)
        .await;

    let err = make_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .unwrap_err();
    assert!(matches!(err, VisorError::AuthError(_)), "got: {err:?}");
}

#[cfg(feature = "phase-contracts")]
#[tokio::test]
async fn http_403_becomes_forbidden_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": { "code": "forbidden", "message": "Key lacks permission" }
        })))
        .mount(&server)
        .await;

    let err = make_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .unwrap_err();
    assert!(matches!(err, VisorError::ForbiddenError(_)), "got: {err:?}");
}

#[cfg(feature = "phase-contracts")]
#[tokio::test]
async fn http_404_becomes_not_found_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings/nonexistent"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": { "code": "not_found", "message": "Listing not found" }
        })))
        .mount(&server)
        .await;

    let err = make_client(server.uri())
        .get_listing("nonexistent", None)
        .await
        .unwrap_err();
    assert!(matches!(err, VisorError::NotFoundError(_)), "got: {err:?}");
}

#[cfg(feature = "phase-contracts")]
#[tokio::test]
async fn http_429_with_integer_retry_after_parses_duration() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "60")
                .set_body_json(json!({
                    "error": { "code": "rate_limited", "message": "Too many requests" }
                })),
        )
        .mount(&server)
        .await;

    let err = make_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .unwrap_err();
    match err {
        VisorError::RateLimitError { retry_after, .. } => {
            assert_eq!(retry_after, Some(std::time::Duration::from_secs(60)));
        }
        other => panic!("expected RateLimitError, got: {other:?}"),
    }
}

#[cfg(feature = "phase-contracts")]
#[tokio::test]
async fn http_429_with_http_date_retry_after_parses_duration() {
    let server = MockServer::start().await;
    // Far-future date ensures max(0, date - now) is positive.
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "Thu, 01 Jan 2099 00:01:00 GMT")
                .set_body_json(json!({
                    "error": { "code": "rate_limited", "message": "Too many requests" }
                })),
        )
        .mount(&server)
        .await;

    let err = make_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .unwrap_err();
    match err {
        VisorError::RateLimitError { retry_after, .. } => {
            let d = retry_after.expect("expected Some(Duration) from HTTP-date Retry-After");
            assert!(
                d.as_secs() > 0,
                "duration should be positive for a future date"
            );
        }
        other => panic!("expected RateLimitError, got: {other:?}"),
    }
}

#[cfg(feature = "phase-contracts")]
#[tokio::test]
async fn http_429_without_retry_after_header_yields_none() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({
            "error": { "code": "rate_limited", "message": "Too many requests" }
        })))
        .mount(&server)
        .await;

    let err = make_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .unwrap_err();
    match err {
        VisorError::RateLimitError { retry_after, .. } => {
            assert_eq!(retry_after, None);
        }
        other => panic!("expected RateLimitError, got: {other:?}"),
    }
}

#[cfg(feature = "phase-contracts")]
#[tokio::test]
async fn http_5xx_becomes_visor_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": { "code": "internal_error", "message": "Something went wrong" }
        })))
        .mount(&server)
        .await;

    let err = make_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .unwrap_err();
    assert!(matches!(err, VisorError::VisorApiError(_)), "got: {err:?}");
}

// ── Malformed error body fallback ─────────────────────────────────────────────

#[cfg(feature = "phase-contracts")]
#[tokio::test]
async fn non_json_error_body_falls_back_to_unknown_error_with_raw_text() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(400).set_body_string("not json at all"))
        .mount(&server)
        .await;

    let err = make_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .unwrap_err();
    match err {
        VisorError::ValidationError(body) => {
            assert_eq!(body.code, "unknown_error");
            assert_eq!(body.message, "not json at all");
        }
        other => panic!("expected ValidationError(unknown_error), got: {other:?}"),
    }
}

#[cfg(feature = "phase-contracts")]
#[tokio::test]
async fn json_without_error_key_falls_back_to_unknown_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "message": "something wrong"   // "error" key missing
        })))
        .mount(&server)
        .await;

    let err = make_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .unwrap_err();
    match err {
        VisorError::ValidationError(body) => {
            assert_eq!(body.code, "unknown_error");
        }
        other => panic!("expected ValidationError(unknown_error), got: {other:?}"),
    }
}

// ── Malformed success body → InvalidResponse (not TransportError) ─────────────
//
// These pass against the stub (which returns InvalidResponse immediately) and
// will continue to pass once Phase 3 transport is in place and actually reads
// the response body.

#[tokio::test]
async fn success_200_with_array_json_becomes_invalid_response() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[1, 2, 3]"))
        .mount(&server)
        .await;

    let err = make_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .unwrap_err();
    assert!(
        matches!(err, VisorError::InvalidResponse { .. }),
        "non-dict JSON body must produce InvalidResponse, got: {err:?}"
    );
}

#[tokio::test]
async fn success_200_with_malformed_json_becomes_invalid_response_not_transport_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{ not valid json"))
        .mount(&server)
        .await;

    let err = make_client(server.uri())
        .filter_listings(&ListingsFilter::default())
        .await
        .unwrap_err();
    // The transport must read raw bytes and map serde failures → InvalidResponse,
    // not rely on response.json() which maps to TransportError via reqwest::Error.
    assert!(
        matches!(err, VisorError::InvalidResponse { .. }),
        "malformed JSON on 200 must be InvalidResponse, got: {err:?}"
    );
}

// ── Serde defaulting tests ────────────────────────────────────────────────────

#[test]
fn listing_summary_photo_urls_absent_defaults_to_empty_vec() {
    use visor::ListingSummary;
    let val = json!({ "id": "abc", "vin": "1HGCM82633A123456" });
    let summary: ListingSummary = serde_json::from_value(val).unwrap();
    assert!(summary.photo_urls.is_empty());
}

#[test]
fn listing_summary_price_history_absent_defaults_to_empty_vec() {
    use visor::ListingSummary;
    let val = json!({ "id": "abc", "vin": "1HGCM82633A123456" });
    let summary: ListingSummary = serde_json::from_value(val).unwrap();
    assert!(summary.price_history.is_empty());
}

#[test]
fn listing_detail_price_history_absent_defaults_to_none() {
    use visor::ListingDetail;
    let val = json!({
        "id": "abc",
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
    });
    let detail: ListingDetail = serde_json::from_value(val).unwrap();
    assert!(detail.price_history.is_none());
}

#[test]
fn listings_page_meta_absent_defaults_to_empty_map() {
    use visor::ListingsPage;
    let val = json!({
        "data": [],
        "pagination": { "limit": 50, "offset": 0, "total": 0 }
    });
    let page: ListingsPage = serde_json::from_value(val).unwrap();
    assert!(page.meta.is_empty());
}

#[test]
fn vehicle_build_window_sticker_verified_absent_defaults_to_false() {
    use visor::VehicleBuild;
    let val = json!({ "year": 2021, "make": "Toyota", "model": "Camry" });
    let build: VehicleBuild = serde_json::from_value(val).unwrap();
    assert!(!build.window_sticker_verified);
}
