pub(crate) mod async_transport;
pub(crate) mod sync_transport;

pub(crate) use async_transport::AsyncVisorTransport;
pub(crate) use sync_transport::SyncVisorTransport;

use std::time::Duration;

use crate::error::{ApiErrorBody, VisorError};

/// Percent-encode a single URL path segment (RFC 3986 unreserved chars pass through unchanged).
pub(crate) fn encode_path_segment(s: &str) -> String {
    s.bytes()
        .fold(String::with_capacity(s.len()), |mut acc, b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
                acc.push(b as char);
            } else {
                acc.push_str(&format!("%{b:02X}"));
            }
            acc
        })
}

fn build_url_with_params(base_url: &str, path: &str, params: &[(String, String)]) -> reqwest::Url {
    let base = format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        path.trim_start_matches('/')
    );
    let mut url = reqwest::Url::parse(&base).expect("transport: invalid base_url");
    if !params.is_empty() {
        url.query_pairs_mut()
            .extend_pairs(params.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    }
    url
}

/// Wraps single-resource responses shaped as `{ "data": T, "meta": {} }`.
/// Used by transport methods that unwrap detail endpoints.
#[derive(serde::Deserialize)]
pub(super) struct DataEnvelope<T> {
    pub(super) data: T,
}

#[derive(serde::Deserialize)]
struct ErrorEnvelope {
    error: Option<ErrorDetail>,
}

#[derive(serde::Deserialize)]
struct ErrorDetail {
    code: Option<String>,
    message: Option<String>,
}

fn parse_error_body(status: u16, bytes: &[u8]) -> ApiErrorBody {
    if let Ok(envelope) = serde_json::from_slice::<ErrorEnvelope>(bytes) {
        if let Some(detail) = envelope.error {
            return ApiErrorBody {
                status,
                code: detail.code.unwrap_or_else(|| "unknown_error".to_string()),
                message: detail.message.unwrap_or_default(),
            };
        }
    }
    ApiErrorBody {
        status,
        code: "unknown_error".to_string(),
        message: String::from_utf8_lossy(bytes).into_owned(),
    }
}

fn map_status_to_error(
    status: u16,
    body: ApiErrorBody,
    retry_after: Option<Duration>,
) -> VisorError {
    match status {
        400 => VisorError::ValidationError(body),
        401 => VisorError::AuthError(body),
        402 => VisorError::PaymentRequiredError(body),
        403 => VisorError::ForbiddenError(body),
        404 => VisorError::NotFoundError(body),
        429 => VisorError::RateLimitError { body, retry_after },
        _ => VisorError::VisorApiError(body),
    }
}

fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    let value = headers.get("Retry-After")?.to_str().ok()?;
    let value = value.trim();

    if let Ok(secs) = value.parse::<u64>() {
        return Some(Duration::from_secs(secs));
    }

    // Try HTTP-date format: "Thu, 01 Jan 2099 00:01:00 GMT"
    if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(value) {
        let future_ts = dt.timestamp();
        let now_ts = chrono::Utc::now().timestamp();
        let secs = (future_ts - now_ts).max(0) as u64;
        return Some(Duration::from_secs(secs));
    }

    None
}
