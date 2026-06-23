use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::error::VisorError;
use crate::models::common::Pagination;
use crate::models::filter_types::{CountryCode, StateCode};

/// Dealer type. Closed vocabulary.
#[derive(Debug, Clone)]
pub enum DealerType {
    Franchise,
    Independent,
}

impl DealerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Franchise => "franchise",
            Self::Independent => "independent",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DealerFilter {
    pub dealer_id: Option<Vec<Uuid>>,
    pub state: Option<Vec<StateCode>>,
    pub country: Option<CountryCode>,
    /// Serializes as wire key `"type"`, not `"dealer_type"`.
    pub dealer_type: Option<DealerType>,
    pub make: Option<Vec<String>>,
    pub q: Option<String>,
    pub limit: u32,
    pub offset: u32,
}

impl Default for DealerFilter {
    fn default() -> Self {
        Self {
            dealer_id: None,
            state: None,
            country: None,
            dealer_type: None,
            make: None,
            q: None,
            limit: 50,
            offset: 0,
        }
    }
}

impl DealerFilter {
    /// Serialize to query-string params.
    pub fn to_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        params.push(("limit".to_string(), self.limit.to_string()));
        params.push(("offset".to_string(), self.offset.to_string()));
        if let Some(ids) = &self.dealer_id {
            if !ids.is_empty() {
                let joined = ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                params.push(("dealer_id".to_string(), joined));
            }
        }
        if let Some(states) = &self.state {
            if !states.is_empty() {
                let joined = states
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                params.push(("state".to_string(), joined));
            }
        }
        if let Some(country) = &self.country {
            params.push(("country".to_string(), country.as_str().to_string()));
        }
        // Wire key is "type", not "dealer_type"
        if let Some(dt) = &self.dealer_type {
            params.push(("type".to_string(), dt.as_str().to_string()));
        }
        if let Some(makes) = &self.make {
            if !makes.is_empty() {
                params.push(("make".to_string(), makes.join(",")));
            }
        }
        if let Some(q) = &self.q {
            params.push(("q".to_string(), q.clone()));
        }
        params
    }

    /// Validate filter constraints before sending a request.
    pub fn validate(&self) -> Result<(), VisorError> {
        if self.limit > 100 {
            return Err(VisorError::InvalidFilter {
                message: format!("limit must be <= 100, got {}", self.limit),
            });
        }
        if let Some(ids) = &self.dealer_id {
            if ids.len() > 100 {
                return Err(VisorError::InvalidFilter {
                    message: format!("dealer_id accepts at most 100 IDs, got {}", ids.len()),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DealerAddress {
    pub line1: Option<String>,
    pub city: String,
    pub state: String,
    pub country: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DealerSummary {
    pub dealer_id: String,
    pub name: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    #[serde(rename = "type")]
    pub dealer_type: String,
    pub website: Option<String>,
    #[serde(default)]
    pub makes: Vec<String>,
    pub listing_count: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DealerDetail {
    pub dealer_id: String,
    pub name: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    #[serde(rename = "type")]
    pub dealer_type: String,
    pub website: Option<String>,
    #[serde(default)]
    pub makes: Vec<String>,
    pub listing_count: i32,
    pub phone: Option<String>,
    pub address: Option<DealerAddress>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DealersPage {
    pub data: Vec<DealerSummary>,
    pub pagination: Pagination,
    #[serde(default)]
    pub meta: HashMap<String, Value>,
}
