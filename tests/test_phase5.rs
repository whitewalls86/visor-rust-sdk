/// Phase 5 contract tests: all stub client methods.
///
/// Tests run with `RUSTFLAGS='--cfg phase_contracts' cargo test`.
/// Sync-client tests create VisorClient *inside* spawn_blocking so that the
/// reqwest blocking runtime is created and dropped entirely in the blocking
/// thread pool, matching the pattern used in test_transport.rs.
#[cfg(phase_contracts)]
mod phase5 {
    use serde_json::json;
    use uuid::Uuid;
    use visor::{
        AsyncVisorClient, ClientConfig, DealerFilter, FacetField, FacetsFilter, ListingsFilter,
        Vin, VisorClient, VisorError,
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

    // ── VIN lookup ────────────────────────────────────────────────────────────

    fn vin_response() -> serde_json::Value {
        json!({
            "data": {
                "vin": "4T1DAACKXTU765422",
                "status": "active",
                "build": {
                    "year": 2026,
                    "make": "Toyota",
                    "model": "Camry",
                    "trim": "SE",
                    "version": null,
                    "body_type": "Sedan",
                    "drivetrain": "FWD",
                    "fuel_type": "Hybrid",
                    "powertrain_type": "HEV",
                    "transmission": "CVT",
                    "engine": "2.5L I4",
                    "cylinders": 4,
                    "doors": 4,
                    "seating_capacity": 5,
                    "exterior_color": null,
                    "interior_color": null,
                    "base_exterior_color": null,
                    "base_interior_color": null,
                    "assembly_location": null,
                    "window_sticker_verified": false,
                    "base_msrp": null,
                    "combined_msrp": null,
                    "options": []
                },
                "latest_listing": null
            },
            "meta": {}
        })
    }

    #[tokio::test]
    async fn lookup_vin_hits_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vins/4T1DAACKXTU765422"))
            .respond_with(ResponseTemplate::new(200).set_body_json(vin_response()))
            .expect(1)
            .mount(&server)
            .await;

        let vin = Vin::new("4T1DAACKXTU765422").unwrap();
        async_client(server.uri())
            .lookup_vin(&vin, None)
            .await
            .expect("lookup_vin should succeed");
    }

    #[tokio::test]
    async fn lookup_vin_sends_include_param() {
        use visor::ListingInclude;
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vins/4T1DAACKXTU765422"))
            .and(query_param("include", "price_history,options"))
            .respond_with(ResponseTemplate::new(200).set_body_json(vin_response()))
            .expect(1)
            .mount(&server)
            .await;

        let vin = Vin::new("4T1DAACKXTU765422").unwrap();
        async_client(server.uri())
            .lookup_vin(
                &vin,
                Some(vec![ListingInclude::PriceHistory, ListingInclude::Options]),
            )
            .await
            .expect("lookup_vin with include should succeed");
    }

    #[tokio::test]
    async fn lookup_vin_unwraps_data_envelope() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vins/4T1DAACKXTU765422"))
            .respond_with(ResponseTemplate::new(200).set_body_json(vin_response()))
            .mount(&server)
            .await;

        let vin = Vin::new("4T1DAACKXTU765422").unwrap();
        let detail = async_client(server.uri())
            .lookup_vin(&vin, None)
            .await
            .unwrap();
        assert_eq!(detail.vin, "4T1DAACKXTU765422");
        assert_eq!(detail.status, "active");
    }

    #[tokio::test]
    async fn lookup_vin_404_becomes_not_found_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vins/4T1DAACKXTU765422"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({
                "error": { "code": "not_found_error", "message": "VIN not found" }
            })))
            .mount(&server)
            .await;

        let vin = Vin::new("4T1DAACKXTU765422").unwrap();
        let err = async_client(server.uri())
            .lookup_vin(&vin, None)
            .await
            .unwrap_err();
        assert!(matches!(err, VisorError::NotFoundError(_)), "got: {err:?}");
    }

    #[tokio::test]
    async fn sync_lookup_vin_hits_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vins/4T1DAACKXTU765422"))
            .respond_with(ResponseTemplate::new(200).set_body_json(vin_response()))
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri();
        tokio::task::spawn_blocking(move || {
            let vin = Vin::new("4T1DAACKXTU765422").unwrap();
            make_sync(server_uri)
                .lookup_vin(&vin, None)
                .expect("sync lookup_vin should succeed");
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn sync_lookup_vin_unwraps_data_envelope() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vins/4T1DAACKXTU765422"))
            .respond_with(ResponseTemplate::new(200).set_body_json(vin_response()))
            .mount(&server)
            .await;

        let server_uri = server.uri();
        let detail = tokio::task::spawn_blocking(move || {
            let vin = Vin::new("4T1DAACKXTU765422").unwrap();
            make_sync(server_uri)
                .lookup_vin(&vin, None)
                .expect("sync lookup_vin should decode envelope")
        })
        .await
        .unwrap();
        assert_eq!(detail.vin, "4T1DAACKXTU765422");
    }

    // ── Vin input validation ──────────────────────────────────────────────────

    #[test]
    fn vin_rejects_too_short() {
        let err = Vin::new("4T1DAACK").unwrap_err();
        assert!(matches!(err, VisorError::InvalidFilter { .. }));
    }

    #[test]
    fn vin_rejects_invalid_char_i() {
        let err = Vin::new("4T1DAACKXTU76542I").unwrap_err();
        assert!(matches!(err, VisorError::InvalidFilter { .. }));
    }

    #[test]
    fn vin_rejects_invalid_char_o() {
        let err = Vin::new("OT1DAACKXTU765422").unwrap_err();
        assert!(matches!(err, VisorError::InvalidFilter { .. }));
    }

    #[test]
    fn vin_rejects_invalid_char_q() {
        let err = Vin::new("4T1DAACKXTU7654Q2").unwrap_err();
        assert!(matches!(err, VisorError::InvalidFilter { .. }));
    }

    #[test]
    fn vin_normalizes_to_uppercase() {
        let vin = Vin::new("4t1daackxtu765422").unwrap();
        assert_eq!(vin.as_str(), "4T1DAACKXTU765422");
    }

    #[test]
    fn vin_accepts_valid_vin() {
        let vin = Vin::new("4T1DAACKXTU765422").unwrap();
        assert_eq!(vin.as_str(), "4T1DAACKXTU765422");
    }

    // ── filter_facets ─────────────────────────────────────────────────────────

    fn facets_response() -> serde_json::Value {
        json!({
            "data": {
                "total": 100,
                "facets": {
                    "make": [{ "value": "Toyota", "count": 100 }]
                },
                "range_facets": {},
                "stats": {}
            },
            "meta": {
                "facets": ["make"],
                "metric": "count",
                "sort": "-count",
                "minimum_metric_count": 10
            }
        })
    }

    #[tokio::test]
    async fn filter_facets_hits_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/facets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(facets_response()))
            .expect(1)
            .mount(&server)
            .await;

        let filter = FacetsFilter::new(vec![FacetField::Make]);
        async_client(server.uri())
            .filter_facets(&filter)
            .await
            .expect("filter_facets should succeed");
    }

    #[tokio::test]
    async fn filter_facets_sends_facets_param() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/facets"))
            .and(query_param("facets", "make,model"))
            .respond_with(ResponseTemplate::new(200).set_body_json(facets_response()))
            .expect(1)
            .mount(&server)
            .await;

        let filter = FacetsFilter::new(vec![FacetField::Make, FacetField::Model]);
        async_client(server.uri())
            .filter_facets(&filter)
            .await
            .expect("filter_facets should include facets param");
    }

    #[tokio::test]
    async fn filter_facets_returns_full_envelope() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/facets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(facets_response()))
            .mount(&server)
            .await;

        let filter = FacetsFilter::new(vec![FacetField::Make]);
        let resp = async_client(server.uri())
            .filter_facets(&filter)
            .await
            .unwrap();
        assert_eq!(resp.data.total, 100);
        assert_eq!(resp.meta.metric, "count");
    }

    #[tokio::test]
    async fn filter_facets_validates_before_request() {
        let err = async_client("http://127.0.0.1:1".to_string())
            .filter_facets(&FacetsFilter::new(vec![]))
            .await
            .unwrap_err();
        assert!(
            matches!(err, VisorError::InvalidFilter { .. }),
            "got: {err:?}"
        );
    }

    #[tokio::test]
    async fn filter_facets_401_becomes_auth_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/facets"))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                "error": { "code": "unauthorized", "message": "bad key" }
            })))
            .mount(&server)
            .await;

        let filter = FacetsFilter::new(vec![FacetField::Make]);
        let err = async_client(server.uri())
            .filter_facets(&filter)
            .await
            .unwrap_err();
        assert!(matches!(err, VisorError::AuthError(_)), "got: {err:?}");
    }

    #[tokio::test]
    async fn sync_filter_facets_hits_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/facets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(facets_response()))
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri();
        tokio::task::spawn_blocking(move || {
            make_sync(server_uri)
                .filter_facets(&FacetsFilter::new(vec![FacetField::Make]))
                .expect("sync filter_facets should succeed");
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn sync_filter_facets_validates_before_request() {
        let err = tokio::task::spawn_blocking(|| {
            make_sync("http://127.0.0.1:1".to_string())
                .filter_facets(&FacetsFilter::new(vec![]))
                .unwrap_err()
        })
        .await
        .unwrap();
        assert!(matches!(err, VisorError::InvalidFilter { .. }));
    }

    // ── search_dealers ────────────────────────────────────────────────────────

    fn dealers_page() -> serde_json::Value {
        json!({
            "data": [
                {
                    "dealer_id": "b62c6042-b3a0-4a58-bc5b-55966bd1c68c",
                    "name": "Claremont Toyota",
                    "city": "Claremont",
                    "state": "CA",
                    "country": "US",
                    "latitude": 34.08,
                    "longitude": -117.72,
                    "type": "franchise",
                    "website": null,
                    "makes": ["Toyota"],
                    "listing_count": 100
                }
            ],
            "pagination": { "limit": 50, "offset": 0, "total": 1, "next_offset": null },
            "meta": {}
        })
    }

    #[tokio::test]
    async fn search_dealers_hits_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/dealers"))
            .respond_with(ResponseTemplate::new(200).set_body_json(dealers_page()))
            .expect(1)
            .mount(&server)
            .await;

        async_client(server.uri())
            .search_dealers(&DealerFilter::default())
            .await
            .expect("search_dealers should succeed");
    }

    #[tokio::test]
    async fn search_dealers_returns_full_page_envelope() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/dealers"))
            .respond_with(ResponseTemplate::new(200).set_body_json(dealers_page()))
            .mount(&server)
            .await;

        let page = async_client(server.uri())
            .search_dealers(&DealerFilter::default())
            .await
            .unwrap();
        assert_eq!(page.data.len(), 1);
        assert_eq!(page.data[0].name, "Claremont Toyota");
        assert_eq!(page.pagination.total, 1);
    }

    #[tokio::test]
    async fn search_dealers_validates_before_request() {
        let mut filter = DealerFilter::default();
        filter.q = Some("   ".to_string());
        let err = async_client("http://127.0.0.1:1".to_string())
            .search_dealers(&filter)
            .await
            .unwrap_err();
        assert!(matches!(err, VisorError::InvalidFilter { .. }));
    }

    #[tokio::test]
    async fn search_dealers_400_becomes_validation_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/dealers"))
            .respond_with(ResponseTemplate::new(400).set_body_json(json!({
                "error": { "code": "validation_error", "message": "bad params" }
            })))
            .mount(&server)
            .await;

        let err = async_client(server.uri())
            .search_dealers(&DealerFilter::default())
            .await
            .unwrap_err();
        assert!(
            matches!(err, VisorError::ValidationError(_)),
            "got: {err:?}"
        );
    }

    #[tokio::test]
    async fn sync_search_dealers_hits_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/dealers"))
            .respond_with(ResponseTemplate::new(200).set_body_json(dealers_page()))
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri();
        tokio::task::spawn_blocking(move || {
            make_sync(server_uri)
                .search_dealers(&DealerFilter::default())
                .expect("sync search_dealers should succeed");
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn sync_search_dealers_validates_before_request() {
        let err = tokio::task::spawn_blocking(|| {
            let mut filter = DealerFilter::default();
            filter.q = Some("   ".to_string());
            make_sync("http://127.0.0.1:1".to_string())
                .search_dealers(&filter)
                .unwrap_err()
        })
        .await
        .unwrap();
        assert!(matches!(err, VisorError::InvalidFilter { .. }));
    }

    // ── get_dealer ────────────────────────────────────────────────────────────

    fn dealer_detail() -> serde_json::Value {
        json!({
            "data": {
                "dealer_id": "b62c6042-b3a0-4a58-bc5b-55966bd1c68c",
                "name": "Claremont Toyota",
                "city": "Claremont",
                "state": "CA",
                "country": "US",
                "latitude": 34.08,
                "longitude": -117.72,
                "type": "franchise",
                "website": null,
                "makes": ["Toyota"],
                "listing_count": 100,
                "phone": "(909) 555-1234",
                "address": {
                    "line1": "601 Auto Center Dr",
                    "city": "Claremont",
                    "state": "CA",
                    "country": "US"
                }
            },
            "meta": {}
        })
    }

    #[tokio::test]
    async fn get_dealer_hits_correct_path() {
        let id = Uuid::parse_str("b62c6042-b3a0-4a58-bc5b-55966bd1c68c").unwrap();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/dealers/b62c6042-b3a0-4a58-bc5b-55966bd1c68c"))
            .respond_with(ResponseTemplate::new(200).set_body_json(dealer_detail()))
            .expect(1)
            .mount(&server)
            .await;

        async_client(server.uri())
            .get_dealer(id)
            .await
            .expect("get_dealer should succeed");
    }

    #[tokio::test]
    async fn get_dealer_unwraps_data_envelope() {
        let id = Uuid::parse_str("b62c6042-b3a0-4a58-bc5b-55966bd1c68c").unwrap();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/dealers/b62c6042-b3a0-4a58-bc5b-55966bd1c68c"))
            .respond_with(ResponseTemplate::new(200).set_body_json(dealer_detail()))
            .mount(&server)
            .await;

        let detail = async_client(server.uri()).get_dealer(id).await.unwrap();
        assert_eq!(detail.name, "Claremont Toyota");
        assert_eq!(detail.phone.as_deref(), Some("(909) 555-1234"));
    }

    #[tokio::test]
    async fn get_dealer_404_becomes_not_found_error() {
        let id = Uuid::parse_str("b62c6042-b3a0-4a58-bc5b-55966bd1c68c").unwrap();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/dealers/b62c6042-b3a0-4a58-bc5b-55966bd1c68c"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({
                "error": { "code": "not_found_error", "message": "Dealer not found" }
            })))
            .mount(&server)
            .await;

        let err = async_client(server.uri()).get_dealer(id).await.unwrap_err();
        assert!(matches!(err, VisorError::NotFoundError(_)), "got: {err:?}");
    }

    #[tokio::test]
    async fn sync_get_dealer_hits_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/dealers/b62c6042-b3a0-4a58-bc5b-55966bd1c68c"))
            .respond_with(ResponseTemplate::new(200).set_body_json(dealer_detail()))
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri();
        tokio::task::spawn_blocking(move || {
            let id = Uuid::parse_str("b62c6042-b3a0-4a58-bc5b-55966bd1c68c").unwrap();
            make_sync(server_uri)
                .get_dealer(id)
                .expect("sync get_dealer should succeed");
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn sync_get_dealer_unwraps_data_envelope() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/dealers/b62c6042-b3a0-4a58-bc5b-55966bd1c68c"))
            .respond_with(ResponseTemplate::new(200).set_body_json(dealer_detail()))
            .mount(&server)
            .await;

        let server_uri = server.uri();
        let detail = tokio::task::spawn_blocking(move || {
            let id = Uuid::parse_str("b62c6042-b3a0-4a58-bc5b-55966bd1c68c").unwrap();
            make_sync(server_uri)
                .get_dealer(id)
                .expect("sync get_dealer should decode envelope")
        })
        .await
        .unwrap();
        assert_eq!(detail.name, "Claremont Toyota");
    }

    // ── dealer_inventory ──────────────────────────────────────────────────────

    fn inventory_page() -> serde_json::Value {
        json!({
            "data": [
                {
                    "id": "abc123",
                    "vin": "3TMLB5JN3TM286572",
                    "year": 2026,
                    "make": "Toyota",
                    "model": "Tacoma"
                }
            ],
            "pagination": { "limit": 50, "offset": 0, "total": 1, "next_offset": null },
            "meta": {}
        })
    }

    #[tokio::test]
    async fn dealer_inventory_hits_correct_path() {
        let id = Uuid::parse_str("b62c6042-b3a0-4a58-bc5b-55966bd1c68c").unwrap();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(
                "/dealers/b62c6042-b3a0-4a58-bc5b-55966bd1c68c/listings",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(inventory_page()))
            .expect(1)
            .mount(&server)
            .await;

        async_client(server.uri())
            .dealer_inventory(id, &ListingsFilter::default())
            .await
            .expect("dealer_inventory should succeed");
    }

    #[tokio::test]
    async fn dealer_inventory_returns_full_page_envelope() {
        let id = Uuid::parse_str("b62c6042-b3a0-4a58-bc5b-55966bd1c68c").unwrap();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(
                "/dealers/b62c6042-b3a0-4a58-bc5b-55966bd1c68c/listings",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(inventory_page()))
            .mount(&server)
            .await;

        let page = async_client(server.uri())
            .dealer_inventory(id, &ListingsFilter::default())
            .await
            .unwrap();
        assert_eq!(page.data.len(), 1);
        assert_eq!(page.data[0].vin, "3TMLB5JN3TM286572");
        assert_eq!(page.pagination.total, 1);
    }

    #[tokio::test]
    async fn dealer_inventory_validates_before_request() {
        let mut filter = ListingsFilter::default();
        filter.limit = 999;
        let id = Uuid::new_v4();
        let err = async_client("http://127.0.0.1:1".to_string())
            .dealer_inventory(id, &filter)
            .await
            .unwrap_err();
        assert!(matches!(err, VisorError::InvalidFilter { .. }));
    }

    #[tokio::test]
    async fn dealer_inventory_401_becomes_auth_error() {
        let id = Uuid::parse_str("b62c6042-b3a0-4a58-bc5b-55966bd1c68c").unwrap();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(
                "/dealers/b62c6042-b3a0-4a58-bc5b-55966bd1c68c/listings",
            ))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                "error": { "code": "unauthorized", "message": "bad key" }
            })))
            .mount(&server)
            .await;

        let err = async_client(server.uri())
            .dealer_inventory(id, &ListingsFilter::default())
            .await
            .unwrap_err();
        assert!(matches!(err, VisorError::AuthError(_)), "got: {err:?}");
    }

    #[tokio::test]
    async fn sync_dealer_inventory_hits_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(
                "/dealers/b62c6042-b3a0-4a58-bc5b-55966bd1c68c/listings",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(inventory_page()))
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri();
        tokio::task::spawn_blocking(move || {
            let id = Uuid::parse_str("b62c6042-b3a0-4a58-bc5b-55966bd1c68c").unwrap();
            make_sync(server_uri)
                .dealer_inventory(id, &ListingsFilter::default())
                .expect("sync dealer_inventory should succeed");
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn sync_dealer_inventory_validates_before_request() {
        let err = tokio::task::spawn_blocking(|| {
            let mut filter = ListingsFilter::default();
            filter.limit = 999;
            let id = Uuid::new_v4();
            make_sync("http://127.0.0.1:1".to_string())
                .dealer_inventory(id, &filter)
                .unwrap_err()
        })
        .await
        .unwrap();
        assert!(matches!(err, VisorError::InvalidFilter { .. }));
    }

    // ── get_usage ─────────────────────────────────────────────────────────────

    fn usage_body() -> serde_json::Value {
        json!({
            "data": [
                {
                    "date": "2024-01-15",
                    "metering_class": "listing_search",
                    "requests": 10,
                    "charged_micros": 20000
                }
            ],
            "totals": { "requests": 10, "charged_micros": 20000 },
            "meta": {
                "start_date": "2024-01-01",
                "end_date": "2024-01-31",
                "interval": "day",
                "currency": "USD",
                "source": "public_api_usage_events",
                "freshness": "eventually consistent"
            }
        })
    }

    #[tokio::test]
    async fn get_usage_hits_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/usage"))
            .respond_with(ResponseTemplate::new(200).set_body_json(usage_body()))
            .expect(1)
            .mount(&server)
            .await;

        async_client(server.uri())
            .get_usage(None, None, None)
            .await
            .expect("get_usage should succeed");
    }

    #[tokio::test]
    async fn get_usage_returns_full_envelope() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/usage"))
            .respond_with(ResponseTemplate::new(200).set_body_json(usage_body()))
            .mount(&server)
            .await;

        let summary = async_client(server.uri())
            .get_usage(None, None, None)
            .await
            .unwrap();
        assert_eq!(summary.data.len(), 1);
        assert_eq!(summary.totals.requests, 10);
        assert_eq!(summary.meta.currency, "USD");
    }

    #[tokio::test]
    async fn get_usage_sends_start_date_and_end_date() {
        use chrono::NaiveDate;
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/usage"))
            .and(query_param("start_date", "2024-01-01"))
            .and(query_param("end_date", "2024-01-31"))
            .respond_with(ResponseTemplate::new(200).set_body_json(usage_body()))
            .expect(1)
            .mount(&server)
            .await;

        async_client(server.uri())
            .get_usage(
                Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
                Some(NaiveDate::from_ymd_opt(2024, 1, 31).unwrap()),
                None,
            )
            .await
            .expect("get_usage with dates should succeed");
    }

    #[tokio::test]
    async fn get_usage_sends_metering_class_comma_separated() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/usage"))
            .and(query_param("metering_class", "listing_search,dealers"))
            .respond_with(ResponseTemplate::new(200).set_body_json(usage_body()))
            .expect(1)
            .mount(&server)
            .await;

        async_client(server.uri())
            .get_usage(
                None,
                None,
                Some(vec!["listing_search".to_string(), "dealers".to_string()]),
            )
            .await
            .expect("get_usage with metering_class should succeed");
    }

    #[tokio::test]
    async fn get_usage_rejects_blank_metering_class_element() {
        let err = async_client("http://127.0.0.1:1".to_string())
            .get_usage(None, None, Some(vec!["  ".to_string()]))
            .await
            .unwrap_err();
        assert!(matches!(err, VisorError::InvalidFilter { .. }));
    }

    #[tokio::test]
    async fn get_usage_omits_empty_metering_class() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/usage"))
            .respond_with(ResponseTemplate::new(200).set_body_json(usage_body()))
            .expect(1)
            .mount(&server)
            .await;

        async_client(server.uri())
            .get_usage(None, None, Some(vec![]))
            .await
            .expect("empty metering_class vec should not send the param");
    }

    #[tokio::test]
    async fn get_usage_401_becomes_auth_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/usage"))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                "error": { "code": "unauthorized", "message": "bad key" }
            })))
            .mount(&server)
            .await;

        let err = async_client(server.uri())
            .get_usage(None, None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, VisorError::AuthError(_)), "got: {err:?}");
    }

    #[tokio::test]
    async fn sync_get_usage_hits_correct_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/usage"))
            .respond_with(ResponseTemplate::new(200).set_body_json(usage_body()))
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri();
        tokio::task::spawn_blocking(move || {
            make_sync(server_uri)
                .get_usage(None, None, None)
                .expect("sync get_usage should succeed");
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn sync_get_usage_returns_full_envelope() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/usage"))
            .respond_with(ResponseTemplate::new(200).set_body_json(usage_body()))
            .mount(&server)
            .await;

        let server_uri = server.uri();
        let summary = tokio::task::spawn_blocking(move || {
            make_sync(server_uri)
                .get_usage(None, None, None)
                .expect("sync get_usage should decode envelope")
        })
        .await
        .unwrap();
        assert_eq!(summary.data.len(), 1);
        assert_eq!(summary.meta.currency, "USD");
    }

    #[tokio::test]
    async fn sync_get_usage_sends_start_date_and_end_date() {
        use chrono::NaiveDate;
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/usage"))
            .and(query_param("start_date", "2024-01-01"))
            .and(query_param("end_date", "2024-01-31"))
            .respond_with(ResponseTemplate::new(200).set_body_json(usage_body()))
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri();
        tokio::task::spawn_blocking(move || {
            make_sync(server_uri)
                .get_usage(
                    Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
                    Some(NaiveDate::from_ymd_opt(2024, 1, 31).unwrap()),
                    None,
                )
                .expect("sync get_usage with dates should succeed");
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn sync_get_usage_rejects_blank_metering_class() {
        let err = tokio::task::spawn_blocking(|| {
            make_sync("http://127.0.0.1:1".to_string())
                .get_usage(None, None, Some(vec!["  ".to_string()]))
                .unwrap_err()
        })
        .await
        .unwrap();
        assert!(matches!(err, VisorError::InvalidFilter { .. }));
    }
}
