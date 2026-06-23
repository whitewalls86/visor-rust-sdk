use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use crate::error::VisorError;
use crate::models::common::Pagination;

#[derive(Debug, Clone)]
pub enum DealerType {
    Franchise,
    Independent,
}

#[derive(Debug, Clone)]
pub struct DealerFilter {
    pub dealer_id: Option<Vec<String>>,
    pub state: Option<Vec<String>>,
    pub country: Option<String>,
    /// Serializes as wire key "type", not "dealer_type".
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
    /// Serialize to query-string params. Stub — Phase 4 TODO.
    pub fn to_params(&self) -> Vec<(String, String)> {
        vec![]
    }

    /// Validate filter constraints before sending a request. Stub — Phase 4 TODO.
    pub fn validate(&self) -> Result<(), VisorError> {
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
