use serde_json::json;
use visor::{AsyncVisorClient, ClientConfig, UsageSummary, VisorError};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_client(base_url: String) -> AsyncVisorClient {
    AsyncVisorClient::with_config(ClientConfig {
        api_key: "test-key".to_string(),
        base_url,
        ..ClientConfig::default()
    })
}

fn usage_body() -> serde_json::Value {
    json!({
        "data": [
            {
                "date": "2024-01-15",
                "metering_class": "listings",
                "requests": 42,
                "charged_micros": 420000
            }
        ],
        "totals": {
            "requests": 42,
            "charged_micros": 420000
        },
        "meta": {
            "start_date": "2024-01-01",
            "end_date": "2024-01-31",
            "interval": "day",
            "currency": "USD",
            "source": "metering",
            "freshness": "realtime"
        }
    })
}

// ── get_usage returns full envelope (data + totals + meta) ────────────────────

#[tokio::test]
async fn get_usage_returns_full_usage_summary_envelope() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/usage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(usage_body()))
        .mount(&server)
        .await;

    let summary = make_client(server.uri())
        .get_usage(None, None, None)
        .await
        .expect("get_usage should succeed");

    assert_eq!(summary.data.len(), 1);
    assert_eq!(summary.data[0].metering_class, "listings");
    assert_eq!(summary.data[0].requests, 42);
    assert_eq!(summary.totals.requests, 42);
    assert_eq!(summary.meta.interval, "day");
}

#[tokio::test]
async fn get_usage_start_and_end_sent_as_iso8601() {
    use chrono::NaiveDate;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/usage"))
        .and(query_param("start", "2024-01-01"))
        .and(query_param("end", "2024-01-31"))
        .respond_with(ResponseTemplate::new(200).set_body_json(usage_body()))
        .mount(&server)
        .await;

    make_client(server.uri())
        .get_usage(
            Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2024, 1, 31).unwrap()),
            None,
        )
        .await
        .expect("get_usage with dates should succeed");
}

#[tokio::test]
async fn get_usage_metering_class_sent_comma_separated() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/usage"))
        .and(query_param("metering_class", "listings,dealers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(usage_body()))
        .mount(&server)
        .await;

    make_client(server.uri())
        .get_usage(
            None,
            None,
            Some(vec!["listings".to_string(), "dealers".to_string()]),
        )
        .await
        .expect("get_usage with metering_class should succeed");
}

// ── Serde defaulting for usage models ────────────────────────────────────────

#[test]
fn usage_summary_deserializes_full_envelope() {
    let summary: UsageSummary = serde_json::from_value(usage_body()).unwrap();
    assert_eq!(summary.data.len(), 1);
    assert_eq!(summary.totals.charged_micros, 420000);
    assert_eq!(summary.meta.currency, "USD");
}

#[test]
fn usage_summary_data_can_be_empty() {
    let val = json!({
        "data": [],
        "totals": { "requests": 0, "charged_micros": 0 },
        "meta": {
            "start_date": "2024-01-01",
            "end_date": "2024-01-31",
            "interval": "day",
            "currency": "USD",
            "source": "metering",
            "freshness": "realtime"
        }
    });
    let summary: UsageSummary = serde_json::from_value(val).unwrap();
    assert!(summary.data.is_empty());
    assert_eq!(summary.totals.requests, 0);
}

// ── Error dispatch for /usage ─────────────────────────────────────────────────

#[tokio::test]
async fn get_usage_401_becomes_auth_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/usage"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": { "code": "unauthorized", "message": "Invalid API key" }
        })))
        .mount(&server)
        .await;

    let err = make_client(server.uri())
        .get_usage(None, None, None)
        .await
        .unwrap_err();
    assert!(matches!(err, VisorError::AuthError(_)), "got: {err:?}");
}
