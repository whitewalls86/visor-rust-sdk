pub mod async_client;
pub mod sync_client;

pub use async_client::AsyncVisorClient;
pub use sync_client::VisorClient;

use chrono::NaiveDate;

use crate::error::VisorError;
use crate::models::base::ListingInclude;

/// Configuration shared by both async and sync clients.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub api_key: String,
    /// Base URL for all requests. Defaults to `https://api.visor.vin/v1`.
    pub base_url: String,
    /// Per-request timeout. Defaults to 30 seconds.
    pub timeout: std::time::Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.visor.vin/v1".to_string(),
            timeout: std::time::Duration::from_secs(30),
        }
    }
}

/// Reject listing IDs that are dot-only path segments.
///
/// `reqwest::Url` follows the WHATWG URL Standard, which normalizes `"."` and `".."` — and
/// their percent-encoded equivalents — into directory traversal regardless of how they are
/// encoded. Real listing IDs from this API are opaque hex strings, so rejecting these values
/// early produces a clear error instead of silently requesting the wrong endpoint.
pub(crate) fn validate_listing_id(id: &str) -> Result<(), VisorError> {
    if id == "." || id == ".." {
        return Err(VisorError::InvalidFilter {
            message: format!(
                "listing ID {id:?} is not a valid identifier; \
                 dot-only path segments are normalized by the URL parser"
            ),
        });
    }
    Ok(())
}

pub(crate) fn build_include_params(include: Option<Vec<ListingInclude>>) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Some(includes) = include {
        if !includes.is_empty() {
            let joined: Vec<&str> = includes.iter().map(|i| i.as_str()).collect();
            params.push(("include".to_string(), joined.join(",")));
        }
    }
    params
}

pub(crate) fn build_usage_params(
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    metering_class: Option<Vec<String>>,
) -> Result<Vec<(String, String)>, VisorError> {
    let mut params = Vec::new();
    if let Some(d) = start {
        params.push(("start_date".to_string(), d.format("%Y-%m-%d").to_string()));
    }
    if let Some(d) = end {
        params.push(("end_date".to_string(), d.format("%Y-%m-%d").to_string()));
    }
    if let Some(classes) = metering_class {
        if !classes.is_empty() {
            for c in &classes {
                if c.trim().is_empty() {
                    return Err(VisorError::InvalidFilter {
                        message: "metering_class contains a blank element".to_string(),
                    });
                }
            }
            params.push(("metering_class".to_string(), classes.join(",")));
        }
    }
    Ok(params)
}
