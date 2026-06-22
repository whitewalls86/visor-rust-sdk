use std::collections::HashMap;

use serde::Deserialize;

use crate::error::VisorError;
use crate::models::base::ListingsFilterBase;

#[derive(Debug, Clone, Default)]
pub enum FacetSort {
    Count,
    #[default]
    CountDesc,
    Metric,
    MetricDesc,
}

pub struct FacetsFilter {
    pub base: ListingsFilterBase,
    pub facets: Vec<String>,
    pub facet_value_limit: Option<u32>,
    pub metric: Option<String>,
    pub sort: FacetSort,
}

impl FacetsFilter {
    /// Construct with required facets; all other fields use defaults.
    pub fn new(facets: Vec<String>) -> Self {
        Self {
            base: ListingsFilterBase::default(),
            facets,
            facet_value_limit: None,
            metric: None,
            sort: FacetSort::CountDesc,
        }
    }

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
pub struct FacetBucket {
    pub value: String,
    pub count: Option<i32>,
    pub metric: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RangeBucket {
    pub min: f64,
    pub max: f64,
    pub count: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RangeFacet {
    pub buckets: Vec<RangeBucket>,
    pub interval: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FieldStats {
    pub min: f64,
    pub max: f64,
    pub count: i32,
    pub missing: i32,
    pub mean: f64,
    pub median: f64,
    pub stddev: f64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FacetsData {
    pub total: i32,
    #[serde(default)]
    pub facets: HashMap<String, Vec<FacetBucket>>,
    #[serde(default)]
    pub range_facets: HashMap<String, RangeFacet>,
    #[serde(default)]
    pub stats: HashMap<String, FieldStats>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FacetsMeta {
    pub facets: Vec<String>,
    pub metric: String,
    pub sort: String,
    pub minimum_metric_count: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FacetsResponse {
    pub data: FacetsData,
    pub meta: FacetsMeta,
}
