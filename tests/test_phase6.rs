use futures::StreamExt;
use serde_json::json;
use visor::{
    AsyncVisorClient, ClientConfig, DealerFilter, ListingsFilter, VisorClient, VisorError,
};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn async_client(base_url: String) -> AsyncVisorClient {
    AsyncVisorClient::with_config(ClientConfig {
        api_key: "test-key".to_string(),
        base_url,
        ..ClientConfig::default()
    })
}

fn make_sync(base_url: String) -> VisorClient {
    VisorClient::with_config(ClientConfig {
        api_key: "test-key".to_string(),
        base_url,
        ..ClientConfig::default()
    })
}

// --- Fixtures ---

fn listing(id: &str) -> serde_json::Value {
    json!({ "id": id, "vin": "TESTVIN00000000000" })
}

fn listings_page(ids: &[&str], offset: i32, next_offset: Option<i32>) -> serde_json::Value {
    let data: Vec<_> = ids.iter().map(|id| listing(id)).collect();
    json!({
        "data": data,
        "pagination": {
            "limit": 2,
            "offset": offset,
            "total": 100,
            "next_offset": next_offset
        },
        "meta": {}
    })
}

fn dealer(id: &str) -> serde_json::Value {
    json!({
        "dealer_id": id,
        "name": format!("Dealer {}", id),
        "city": "Austin",
        "state": "TX",
        "country": "US",
        "type": "franchise",
        "listing_count": 5
    })
}

fn dealers_page(ids: &[&str], offset: i32, next_offset: Option<i32>) -> serde_json::Value {
    let data: Vec<_> = ids.iter().map(|id| dealer(id)).collect();
    json!({
        "data": data,
        "pagination": {
            "limit": 2,
            "offset": offset,
            "total": 100,
            "next_offset": next_offset
        },
        "meta": {}
    })
}

// --- Async listings ---

#[tokio::test]
async fn paginate_listings_multiple_pages_flattened_in_order() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .and(query_param("offset", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(
            &["a", "b"],
            0,
            Some(2),
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .and(query_param("offset", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(&["c", "d"], 2, None)))
        .expect(1)
        .mount(&server)
        .await;

    let client = async_client(server.uri());
    let items: Vec<_> = visor::paginate_listings(&client, ListingsFilter::default(), None)
        .collect()
        .await;

    let ids: Vec<_> = items.into_iter().map(|r| r.unwrap().id).collect();
    assert_eq!(ids, vec!["a", "b", "c", "d"]);
}

#[tokio::test]
async fn paginate_listings_propagates_next_offset() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .and(query_param("offset", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(&["a"], 0, Some(50))))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .and(query_param("offset", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(&["b"], 50, None)))
        .expect(1)
        .mount(&server)
        .await;

    let client = async_client(server.uri());
    let items: Vec<_> = visor::paginate_listings(&client, ListingsFilter::default(), None)
        .collect()
        .await;

    assert_eq!(items.len(), 2);
    // server expectations verified on drop
}

#[tokio::test]
async fn paginate_listings_max_pages_zero_makes_no_request() {
    let server = MockServer::start().await;
    // No mock mounted -- any request would cause wiremock to error.

    let client = async_client(server.uri());
    let items: Vec<_> = visor::paginate_listings(&client, ListingsFilter::default(), Some(0))
        .collect()
        .await;

    assert!(items.is_empty());
    assert_eq!(
        server.received_requests().await.unwrap().len(),
        0,
        "Some(0) must perform no request"
    );
}

#[tokio::test]
async fn paginate_listings_max_pages_one_stops_after_first_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .and(query_param("offset", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(
            &["a", "b"],
            0,
            Some(2),
        )))
        .expect(1)
        .mount(&server)
        .await;

    let client = async_client(server.uri());
    let items: Vec<_> = visor::paginate_listings(&client, ListingsFilter::default(), Some(1))
        .collect()
        .await;

    let ids: Vec<_> = items.into_iter().map(|r| r.unwrap().id).collect();
    assert_eq!(ids, vec!["a", "b"]);
    // expect(1) on the mock verifies exactly one request on drop
}

#[tokio::test]
async fn paginate_listings_stops_on_empty_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "pagination": {"limit": 50, "offset": 0, "total": 0, "next_offset": null},
            "meta": {}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = async_client(server.uri());
    let items: Vec<_> = visor::paginate_listings(&client, ListingsFilter::default(), None)
        .collect()
        .await;

    assert!(items.is_empty());
}

#[tokio::test]
async fn paginate_listings_stops_when_next_offset_null() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(&["a", "b"], 0, None)))
        .expect(1)
        .mount(&server)
        .await;

    let client = async_client(server.uri());
    let items: Vec<_> = visor::paginate_listings(&client, ListingsFilter::default(), None)
        .collect()
        .await;

    let ids: Vec<_> = items.into_iter().map(|r| r.unwrap().id).collect();
    assert_eq!(ids, vec!["a", "b"]);
    // expect(1) ensures no second request was made
}

#[tokio::test]
async fn paginate_listings_error_propagated_and_stream_terminates() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": {"code": "server_error", "message": "internal error"}
        })))
        .mount(&server)
        .await;

    let client = async_client(server.uri());
    let items: Vec<_> = visor::paginate_listings(&client, ListingsFilter::default(), None)
        .collect()
        .await;

    assert_eq!(items.len(), 1);
    assert!(items[0].is_err());
    assert_eq!(
        server.received_requests().await.unwrap().len(),
        1,
        "stream must terminate after one error"
    );
}

#[tokio::test]
async fn paginate_listings_preserves_initial_filter_offset() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .and(query_param("offset", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(
            &["x"],
            100,
            Some(150),
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .and(query_param("offset", "150"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(&["y"], 150, None)))
        .expect(1)
        .mount(&server)
        .await;

    let filter = ListingsFilter {
        offset: 100,
        ..Default::default()
    };

    let client = async_client(server.uri());
    let items: Vec<_> = visor::paginate_listings(&client, filter, None)
        .collect()
        .await;

    let ids: Vec<_> = items.into_iter().map(|r| r.unwrap().id).collect();
    assert_eq!(ids, vec!["x", "y"]);
}

#[tokio::test]
async fn paginate_listings_stops_on_non_advancing_next_offset() {
    let server = MockServer::start().await;
    // next_offset equals current offset -- non-advancing, must not loop.
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{"id": "a", "vin": "TESTVIN00000000000"}],
            "pagination": {"limit": 50, "offset": 0, "total": 100, "next_offset": 0},
            "meta": {}
        })))
        .mount(&server)
        .await;

    let client = async_client(server.uri());
    let items: Vec<_> = visor::paginate_listings(&client, ListingsFilter::default(), None)
        .collect()
        .await;

    let ids: Vec<_> = items.into_iter().map(|r| r.unwrap().id).collect();
    assert_eq!(ids, vec!["a"]);
    assert_eq!(
        server.received_requests().await.unwrap().len(),
        1,
        "non-advancing next_offset must stop pagination after one page"
    );
}

// --- Async dealers ---

#[tokio::test]
async fn paginate_dealers_multiple_pages_flattened_in_order() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dealers"))
        .and(query_param("offset", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(dealers_page(
            &["d1", "d2"],
            0,
            Some(2),
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/dealers"))
        .and(query_param("offset", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(dealers_page(&["d3"], 2, None)))
        .expect(1)
        .mount(&server)
        .await;

    let client = async_client(server.uri());
    let items: Vec<_> = visor::paginate_dealers(&client, DealerFilter::default(), None)
        .collect()
        .await;

    let ids: Vec<_> = items.into_iter().map(|r| r.unwrap().dealer_id).collect();
    assert_eq!(ids, vec!["d1", "d2", "d3"]);
}

#[tokio::test]
async fn paginate_dealers_max_pages_zero_makes_no_request() {
    let server = MockServer::start().await;

    let client = async_client(server.uri());
    let items: Vec<_> = visor::paginate_dealers(&client, DealerFilter::default(), Some(0))
        .collect()
        .await;

    assert!(items.is_empty());
    assert_eq!(server.received_requests().await.unwrap().len(), 0);
}

#[tokio::test]
async fn paginate_dealers_error_propagated_and_stream_terminates() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dealers"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": {"code": "auth_error", "message": "Unauthorized"}
        })))
        .mount(&server)
        .await;

    let client = async_client(server.uri());
    let items: Vec<_> = visor::paginate_dealers(&client, DealerFilter::default(), None)
        .collect()
        .await;

    assert_eq!(items.len(), 1);
    assert!(matches!(items[0], Err(VisorError::AuthError { .. })));
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn paginate_dealers_stops_on_non_advancing_next_offset() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dealers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [dealer("d1")],
            "pagination": {"limit": 50, "offset": 0, "total": 100, "next_offset": 0},
            "meta": {}
        })))
        .mount(&server)
        .await;

    let client = async_client(server.uri());
    let items: Vec<_> = visor::paginate_dealers(&client, DealerFilter::default(), None)
        .collect()
        .await;

    assert_eq!(items.len(), 1);
    assert!(items[0].is_ok());
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

// --- Sync listings ---

#[tokio::test]
async fn iter_listings_multiple_pages_flattened_in_order() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .and(query_param("offset", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(
            &["a", "b"],
            0,
            Some(2),
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .and(query_param("offset", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(&["c"], 2, None)))
        .expect(1)
        .mount(&server)
        .await;

    let base_url = server.uri();
    let ids = tokio::task::spawn_blocking(move || {
        let client = make_sync(base_url);
        visor::iter_listings(&client, ListingsFilter::default(), None)
            .map(|r| r.unwrap().id)
            .collect::<Vec<_>>()
    })
    .await
    .unwrap();

    assert_eq!(ids, vec!["a", "b", "c"]);
}

#[tokio::test]
async fn iter_listings_propagates_next_offset() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .and(query_param("offset", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(&["a"], 0, Some(50))))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .and(query_param("offset", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(&["b"], 50, None)))
        .expect(1)
        .mount(&server)
        .await;

    let base_url = server.uri();
    let count = tokio::task::spawn_blocking(move || {
        let client = make_sync(base_url);
        visor::iter_listings(&client, ListingsFilter::default(), None).count()
    })
    .await
    .unwrap();

    assert_eq!(count, 2);
}

#[tokio::test]
async fn iter_listings_max_pages_zero_makes_no_request() {
    let server = MockServer::start().await;
    let base_url = server.uri();

    let items = tokio::task::spawn_blocking(move || {
        let client = make_sync(base_url);
        visor::iter_listings(&client, ListingsFilter::default(), Some(0)).collect::<Vec<_>>()
    })
    .await
    .unwrap();

    assert!(items.is_empty());
    assert_eq!(server.received_requests().await.unwrap().len(), 0);
}

#[tokio::test]
async fn iter_listings_max_pages_one_stops_after_first_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(
            &["a", "b"],
            0,
            Some(2),
        )))
        .expect(1)
        .mount(&server)
        .await;

    let base_url = server.uri();
    let ids = tokio::task::spawn_blocking(move || {
        let client = make_sync(base_url);
        visor::iter_listings(&client, ListingsFilter::default(), Some(1))
            .map(|r| r.unwrap().id)
            .collect::<Vec<_>>()
    })
    .await
    .unwrap();

    assert_eq!(ids, vec!["a", "b"]);
}

#[tokio::test]
async fn iter_listings_stops_on_empty_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "pagination": {"limit": 50, "offset": 0, "total": 0, "next_offset": null},
            "meta": {}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let base_url = server.uri();
    let items = tokio::task::spawn_blocking(move || {
        let client = make_sync(base_url);
        visor::iter_listings(&client, ListingsFilter::default(), None).collect::<Vec<_>>()
    })
    .await
    .unwrap();

    assert!(items.is_empty());
}

#[tokio::test]
async fn iter_listings_stops_when_next_offset_null() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(&["a", "b"], 0, None)))
        .expect(1)
        .mount(&server)
        .await;

    let base_url = server.uri();
    let ids = tokio::task::spawn_blocking(move || {
        let client = make_sync(base_url);
        visor::iter_listings(&client, ListingsFilter::default(), None)
            .map(|r| r.unwrap().id)
            .collect::<Vec<_>>()
    })
    .await
    .unwrap();

    assert_eq!(ids, vec!["a", "b"]);
}

#[tokio::test]
async fn iter_listings_error_propagated_and_iterator_terminates() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": {"code": "server_error", "message": "internal error"}
        })))
        .mount(&server)
        .await;

    let base_url = server.uri();
    let items = tokio::task::spawn_blocking(move || {
        let client = make_sync(base_url);
        visor::iter_listings(&client, ListingsFilter::default(), None).collect::<Vec<_>>()
    })
    .await
    .unwrap();

    assert_eq!(items.len(), 1);
    assert!(items[0].is_err());
}

#[tokio::test]
async fn iter_listings_preserves_initial_filter_offset() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .and(query_param("offset", "200"))
        .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(&["x"], 200, None)))
        .expect(1)
        .mount(&server)
        .await;

    let filter = ListingsFilter {
        offset: 200,
        ..Default::default()
    };

    let base_url = server.uri();
    let ids = tokio::task::spawn_blocking(move || {
        let client = make_sync(base_url);
        visor::iter_listings(&client, filter, None)
            .map(|r| r.unwrap().id)
            .collect::<Vec<_>>()
    })
    .await
    .unwrap();

    assert_eq!(ids, vec!["x"]);
}

#[tokio::test]
async fn iter_listings_stops_on_non_advancing_next_offset() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{"id": "a", "vin": "TESTVIN00000000000"}],
            "pagination": {"limit": 50, "offset": 0, "total": 100, "next_offset": 0},
            "meta": {}
        })))
        .mount(&server)
        .await;

    let base_url = server.uri();
    let ids = tokio::task::spawn_blocking(move || {
        let client = make_sync(base_url);
        visor::iter_listings(&client, ListingsFilter::default(), None)
            .map(|r| r.unwrap().id)
            .collect::<Vec<_>>()
    })
    .await
    .unwrap();

    assert_eq!(ids, vec!["a"]);
    assert_eq!(
        server.received_requests().await.unwrap().len(),
        1,
        "non-advancing next_offset must stop after one page"
    );
}

// --- Sync dealers ---

#[tokio::test]
async fn iter_dealers_multiple_pages_flattened_in_order() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dealers"))
        .and(query_param("offset", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(dealers_page(
            &["d1", "d2"],
            0,
            Some(2),
        )))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/dealers"))
        .and(query_param("offset", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(dealers_page(&["d3"], 2, None)))
        .expect(1)
        .mount(&server)
        .await;

    let base_url = server.uri();
    let ids = tokio::task::spawn_blocking(move || {
        let client = make_sync(base_url);
        visor::iter_dealers(&client, DealerFilter::default(), None)
            .map(|r| r.unwrap().dealer_id)
            .collect::<Vec<_>>()
    })
    .await
    .unwrap();

    assert_eq!(ids, vec!["d1", "d2", "d3"]);
}

#[tokio::test]
async fn iter_dealers_max_pages_zero_makes_no_request() {
    let server = MockServer::start().await;
    let base_url = server.uri();

    let items = tokio::task::spawn_blocking(move || {
        let client = make_sync(base_url);
        visor::iter_dealers(&client, DealerFilter::default(), Some(0)).collect::<Vec<_>>()
    })
    .await
    .unwrap();

    assert!(items.is_empty());
    assert_eq!(server.received_requests().await.unwrap().len(), 0);
}

#[tokio::test]
async fn iter_dealers_error_propagated_and_iterator_terminates() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dealers"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({
            "error": {"code": "rate_limit_error", "message": "Too many requests"}
        })))
        .mount(&server)
        .await;

    let base_url = server.uri();
    let items = tokio::task::spawn_blocking(move || {
        let client = make_sync(base_url);
        visor::iter_dealers(&client, DealerFilter::default(), None).collect::<Vec<_>>()
    })
    .await
    .unwrap();

    assert_eq!(items.len(), 1);
    assert!(items[0].is_err());
}

// --- Repeated-polling after exhaustion ---

#[tokio::test]
async fn iter_listings_repeated_next_after_empty_page_makes_no_extra_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "pagination": {"limit": 50, "offset": 0, "total": 0, "next_offset": null},
            "meta": {}
        })))
        .mount(&server)
        .await;

    let base_url = server.uri();
    tokio::task::spawn_blocking(move || {
        let client = make_sync(base_url);
        let mut iter = visor::iter_listings(&client, ListingsFilter::default(), None);
        assert!(iter.next().is_none());
        assert!(iter.next().is_none()); // must not trigger a second request
    })
    .await
    .unwrap();

    assert_eq!(
        server.received_requests().await.unwrap().len(),
        1,
        "repeated next() after empty-page exhaustion must not send extra requests"
    );
}

#[tokio::test]
async fn iter_dealers_repeated_next_after_empty_page_makes_no_extra_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dealers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "pagination": {"limit": 50, "offset": 0, "total": 0, "next_offset": null},
            "meta": {}
        })))
        .mount(&server)
        .await;

    let base_url = server.uri();
    tokio::task::spawn_blocking(move || {
        let client = make_sync(base_url);
        let mut iter = visor::iter_dealers(&client, DealerFilter::default(), None);
        assert!(iter.next().is_none());
        assert!(iter.next().is_none()); // must not trigger a second request
    })
    .await
    .unwrap();

    assert_eq!(
        server.received_requests().await.unwrap().len(),
        1,
        "repeated next() after empty-page exhaustion must not send extra requests"
    );
}
