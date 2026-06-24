/// Phase 6.5 contract tests: dealer-inventory pagination helpers.
///
/// Run with `RUSTFLAGS='--cfg phase_contracts' cargo test`.
/// Sync-client tests create VisorClient inside spawn_blocking so that the
/// reqwest blocking runtime is created and dropped entirely in the blocking
/// thread pool, matching the pattern used in test_phase6.rs.
#[cfg(phase_contracts)]
mod phase6_5 {
    use futures::StreamExt;
    use serde_json::json;
    use uuid::Uuid;
    use visor::{AsyncVisorClient, ClientConfig, ListingsFilter, VisorClient};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const DEALER_UUID: &str = "00000000-0000-0000-0000-000000000001";

    fn dealer_uuid() -> Uuid {
        DEALER_UUID.parse().unwrap()
    }

    fn dealer_inventory_path() -> String {
        format!("/dealers/{}/listings", DEALER_UUID)
    }

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

    // ── Async dealer inventory ────────────────────────────────────────────────

    #[tokio::test]
    async fn paginate_dealer_inventory_hits_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
            .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(&["a"], 0, None)))
            .expect(1)
            .mount(&server)
            .await;

        let client = async_client(server.uri());
        let items: Vec<_> = visor::paginate_dealer_inventory(
            &client,
            dealer_uuid(),
            ListingsFilter::default(),
            None,
        )
        .collect()
        .await;

        assert_eq!(items.len(), 1);
        assert!(items[0].is_ok());
    }

    #[tokio::test]
    async fn paginate_dealer_inventory_multiple_pages_flattened_in_order() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
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
            .and(path(dealer_inventory_path()))
            .and(query_param("offset", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(
                &["c", "d"],
                2,
                None,
            )))
            .expect(1)
            .mount(&server)
            .await;

        let client = async_client(server.uri());
        let items: Vec<_> = visor::paginate_dealer_inventory(
            &client,
            dealer_uuid(),
            ListingsFilter::default(),
            None,
        )
        .collect()
        .await;

        let ids: Vec<_> = items.into_iter().map(|r| r.unwrap().id).collect();
        assert_eq!(ids, vec!["a", "b", "c", "d"]);
    }

    #[tokio::test]
    async fn paginate_dealer_inventory_propagates_next_offset() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(
                &["a"],
                0,
                Some(50),
            )))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
            .and(query_param("offset", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(&["b"], 50, None)))
            .expect(1)
            .mount(&server)
            .await;

        let client = async_client(server.uri());
        let items: Vec<_> = visor::paginate_dealer_inventory(
            &client,
            dealer_uuid(),
            ListingsFilter::default(),
            None,
        )
        .collect()
        .await;

        assert_eq!(items.len(), 2);
    }

    #[tokio::test]
    async fn paginate_dealer_inventory_preserves_initial_filter_offset() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
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
            .and(path(dealer_inventory_path()))
            .and(query_param("offset", "150"))
            .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(
                &["y"],
                150,
                None,
            )))
            .expect(1)
            .mount(&server)
            .await;

        let mut filter = ListingsFilter::default();
        filter.offset = 100;

        let client = async_client(server.uri());
        let items: Vec<_> = visor::paginate_dealer_inventory(&client, dealer_uuid(), filter, None)
            .collect()
            .await;

        let ids: Vec<_> = items.into_iter().map(|r| r.unwrap().id).collect();
        assert_eq!(ids, vec!["x", "y"]);
    }

    #[tokio::test]
    async fn paginate_dealer_inventory_max_pages_zero_makes_no_request() {
        let server = MockServer::start().await;

        let client = async_client(server.uri());
        let items: Vec<_> = visor::paginate_dealer_inventory(
            &client,
            dealer_uuid(),
            ListingsFilter::default(),
            Some(0),
        )
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
    async fn paginate_dealer_inventory_max_pages_one_stops_after_first_page() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
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
        let items: Vec<_> = visor::paginate_dealer_inventory(
            &client,
            dealer_uuid(),
            ListingsFilter::default(),
            Some(1),
        )
        .collect()
        .await;

        let ids: Vec<_> = items.into_iter().map(|r| r.unwrap().id).collect();
        assert_eq!(ids, vec!["a", "b"]);
    }

    #[tokio::test]
    async fn paginate_dealer_inventory_stops_on_empty_page() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [],
                "pagination": {"limit": 50, "offset": 0, "total": 0, "next_offset": null},
                "meta": {}
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = async_client(server.uri());
        let items: Vec<_> = visor::paginate_dealer_inventory(
            &client,
            dealer_uuid(),
            ListingsFilter::default(),
            None,
        )
        .collect()
        .await;

        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn paginate_dealer_inventory_stops_on_terminal_page_null_next_offset() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
            .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(
                &["a", "b"],
                0,
                None,
            )))
            .expect(1)
            .mount(&server)
            .await;

        let client = async_client(server.uri());
        let items: Vec<_> = visor::paginate_dealer_inventory(
            &client,
            dealer_uuid(),
            ListingsFilter::default(),
            None,
        )
        .collect()
        .await;

        let ids: Vec<_> = items.into_iter().map(|r| r.unwrap().id).collect();
        assert_eq!(ids, vec!["a", "b"]);
    }

    #[tokio::test]
    async fn paginate_dealer_inventory_stops_on_non_advancing_next_offset() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [listing("a")],
                "pagination": {"limit": 50, "offset": 0, "total": 100, "next_offset": 0},
                "meta": {}
            })))
            .mount(&server)
            .await;

        let client = async_client(server.uri());
        let items: Vec<_> = visor::paginate_dealer_inventory(
            &client,
            dealer_uuid(),
            ListingsFilter::default(),
            None,
        )
        .collect()
        .await;

        let ids: Vec<_> = items.into_iter().map(|r| r.unwrap().id).collect();
        assert_eq!(ids, vec!["a"]);
        assert_eq!(
            server.received_requests().await.unwrap().len(),
            1,
            "non-advancing next_offset must stop after one page"
        );
    }

    #[tokio::test]
    async fn paginate_dealer_inventory_error_propagated_and_stream_terminates() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
            .respond_with(ResponseTemplate::new(500).set_body_json(json!({
                "error": {"code": "server_error", "message": "internal error"}
            })))
            .mount(&server)
            .await;

        let client = async_client(server.uri());
        let items: Vec<_> = visor::paginate_dealer_inventory(
            &client,
            dealer_uuid(),
            ListingsFilter::default(),
            None,
        )
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

    // ── Sync dealer inventory ─────────────────────────────────────────────────

    #[tokio::test]
    async fn iter_dealer_inventory_hits_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
            .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(&["a"], 0, None)))
            .expect(1)
            .mount(&server)
            .await;

        let base_url = server.uri();
        let ids = tokio::task::spawn_blocking(move || {
            let client = make_sync(base_url);
            visor::iter_dealer_inventory(&client, dealer_uuid(), ListingsFilter::default(), None)
                .map(|r| r.unwrap().id)
                .collect::<Vec<_>>()
        })
        .await
        .unwrap();

        assert_eq!(ids, vec!["a"]);
    }

    #[tokio::test]
    async fn iter_dealer_inventory_multiple_pages_flattened_in_order() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
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
            .and(path(dealer_inventory_path()))
            .and(query_param("offset", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(&["c"], 2, None)))
            .expect(1)
            .mount(&server)
            .await;

        let base_url = server.uri();
        let ids = tokio::task::spawn_blocking(move || {
            let client = make_sync(base_url);
            visor::iter_dealer_inventory(&client, dealer_uuid(), ListingsFilter::default(), None)
                .map(|r| r.unwrap().id)
                .collect::<Vec<_>>()
        })
        .await
        .unwrap();

        assert_eq!(ids, vec!["a", "b", "c"]);
    }

    #[tokio::test]
    async fn iter_dealer_inventory_propagates_next_offset() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(
                &["a"],
                0,
                Some(50),
            )))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
            .and(query_param("offset", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(&["b"], 50, None)))
            .expect(1)
            .mount(&server)
            .await;

        let base_url = server.uri();
        let count = tokio::task::spawn_blocking(move || {
            let client = make_sync(base_url);
            visor::iter_dealer_inventory(&client, dealer_uuid(), ListingsFilter::default(), None)
                .count()
        })
        .await
        .unwrap();

        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn iter_dealer_inventory_preserves_initial_filter_offset() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
            .and(query_param("offset", "200"))
            .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(
                &["x"],
                200,
                None,
            )))
            .expect(1)
            .mount(&server)
            .await;

        let mut filter = ListingsFilter::default();
        filter.offset = 200;

        let base_url = server.uri();
        let ids = tokio::task::spawn_blocking(move || {
            let client = make_sync(base_url);
            visor::iter_dealer_inventory(&client, dealer_uuid(), filter, None)
                .map(|r| r.unwrap().id)
                .collect::<Vec<_>>()
        })
        .await
        .unwrap();

        assert_eq!(ids, vec!["x"]);
    }

    #[tokio::test]
    async fn iter_dealer_inventory_max_pages_zero_makes_no_request() {
        let server = MockServer::start().await;
        let base_url = server.uri();

        let items = tokio::task::spawn_blocking(move || {
            let client = make_sync(base_url);
            visor::iter_dealer_inventory(&client, dealer_uuid(), ListingsFilter::default(), Some(0))
                .collect::<Vec<_>>()
        })
        .await
        .unwrap();

        assert!(items.is_empty());
        assert_eq!(server.received_requests().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn iter_dealer_inventory_max_pages_one_stops_after_first_page() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
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
            visor::iter_dealer_inventory(&client, dealer_uuid(), ListingsFilter::default(), Some(1))
                .map(|r| r.unwrap().id)
                .collect::<Vec<_>>()
        })
        .await
        .unwrap();

        assert_eq!(ids, vec!["a", "b"]);
    }

    #[tokio::test]
    async fn iter_dealer_inventory_stops_on_empty_page() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
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
            visor::iter_dealer_inventory(&client, dealer_uuid(), ListingsFilter::default(), None)
                .collect::<Vec<_>>()
        })
        .await
        .unwrap();

        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn iter_dealer_inventory_stops_on_terminal_page_null_next_offset() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
            .respond_with(ResponseTemplate::new(200).set_body_json(listings_page(
                &["a", "b"],
                0,
                None,
            )))
            .expect(1)
            .mount(&server)
            .await;

        let base_url = server.uri();
        let ids = tokio::task::spawn_blocking(move || {
            let client = make_sync(base_url);
            visor::iter_dealer_inventory(&client, dealer_uuid(), ListingsFilter::default(), None)
                .map(|r| r.unwrap().id)
                .collect::<Vec<_>>()
        })
        .await
        .unwrap();

        assert_eq!(ids, vec!["a", "b"]);
    }

    #[tokio::test]
    async fn iter_dealer_inventory_stops_on_non_advancing_next_offset() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [listing("a")],
                "pagination": {"limit": 50, "offset": 0, "total": 100, "next_offset": 0},
                "meta": {}
            })))
            .mount(&server)
            .await;

        let base_url = server.uri();
        let ids = tokio::task::spawn_blocking(move || {
            let client = make_sync(base_url);
            visor::iter_dealer_inventory(&client, dealer_uuid(), ListingsFilter::default(), None)
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

    #[tokio::test]
    async fn iter_dealer_inventory_error_propagated_and_iterator_terminates() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
            .respond_with(ResponseTemplate::new(500).set_body_json(json!({
                "error": {"code": "server_error", "message": "internal error"}
            })))
            .mount(&server)
            .await;

        let base_url = server.uri();
        let items = tokio::task::spawn_blocking(move || {
            let client = make_sync(base_url);
            visor::iter_dealer_inventory(&client, dealer_uuid(), ListingsFilter::default(), None)
                .collect::<Vec<_>>()
        })
        .await
        .unwrap();

        assert_eq!(items.len(), 1);
        assert!(items[0].is_err());
    }

    #[tokio::test]
    async fn iter_dealer_inventory_repeated_next_after_exhaustion_makes_no_extra_request() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(dealer_inventory_path()))
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
            let mut iter = visor::iter_dealer_inventory(
                &client,
                dealer_uuid(),
                ListingsFilter::default(),
                None,
            );
            assert!(iter.next().is_none());
            assert!(iter.next().is_none()); // must not trigger a second request
        })
        .await
        .unwrap();

        assert_eq!(
            server.received_requests().await.unwrap().len(),
            1,
            "repeated next() after exhaustion must not send extra requests"
        );
    }
}
