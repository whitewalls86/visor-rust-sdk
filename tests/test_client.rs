use serde_json::json;
use visor::{AsyncVisorClient, ClientConfig, ListingsFilter, VisorClient, VisorError};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Serialize all tests that mutate VISOR_API_KEY so they can't race.
static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

// ── Constructor contracts ─────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "api_key must not be empty")]
fn async_client_new_with_empty_key_panics() {
    AsyncVisorClient::new(String::new());
}

#[test]
#[should_panic(expected = "api_key must not be empty")]
fn sync_client_new_with_empty_key_panics() {
    VisorClient::new(String::new());
}

#[test]
fn async_from_env_without_env_var_returns_missing_api_key() {
    let _guard = ENV_MUTEX.lock().unwrap();
    std::env::remove_var("VISOR_API_KEY");
    let result = AsyncVisorClient::from_env();
    assert!(
        matches!(result, Err(VisorError::MissingApiKey)),
        "got: {result:?}"
    );
}

#[test]
fn sync_from_env_without_env_var_returns_missing_api_key() {
    let _guard = ENV_MUTEX.lock().unwrap();
    std::env::remove_var("VISOR_API_KEY");
    let result = VisorClient::from_env();
    assert!(
        matches!(result, Err(VisorError::MissingApiKey)),
        "got: {result:?}"
    );
}

#[test]
fn client_config_default_base_url_is_production_url() {
    let config = ClientConfig::default();
    assert_eq!(config.base_url, "https://api.visor.vin/v1");
}

#[test]
fn client_config_default_timeout_is_30_seconds() {
    let config = ClientConfig::default();
    assert_eq!(config.timeout, std::time::Duration::from_secs(30));
}

// ── Base URL composition ──────────────────────────────────────────────────────
//
// Transport must concatenate as:
//   base_url.trim_end_matches('/') + "/" + path.trim_start_matches('/')
// Not Url::join — that drops path segments (e.g. /v1 would be lost in production).

#[tokio::test]
#[ignore = "Phase 3: transport not yet implemented"]
async fn base_url_with_trailing_slash_still_hits_correct_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "pagination": { "limit": 50, "offset": 0, "total": 0 }
        })))
        .expect(1)
        .mount(&server)
        .await;

    // Add trailing slash to the base URL — transport must strip it before joining.
    let base_url = format!("{}/", server.uri());
    let client = AsyncVisorClient::with_config(ClientConfig {
        api_key: "test-key".to_string(),
        base_url,
        ..ClientConfig::default()
    });
    client
        .filter_listings(&ListingsFilter::default())
        .await
        .expect("request should succeed");
    // MockServer verifies expect(1) was satisfied on drop.
}

#[tokio::test]
#[ignore = "Phase 3: transport not yet implemented"]
async fn base_url_without_trailing_slash_also_composes_correctly() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/listings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "pagination": { "limit": 50, "offset": 0, "total": 0 }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = AsyncVisorClient::with_config(ClientConfig {
        api_key: "test-key".to_string(),
        base_url: server.uri(),
        ..ClientConfig::default()
    });
    client
        .filter_listings(&ListingsFilter::default())
        .await
        .expect("request should succeed");
}
