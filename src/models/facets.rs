use std::collections::HashMap;

use serde::Deserialize;

use crate::error::VisorError;
use crate::models::base::ListingsFilterBase;

// ── FacetField ────────────────────────────────────────────────────────────────

/// A facet field supported by the `/v1/facets` endpoint.
///
/// Categorical facets produce value-count buckets. Numeric-range facets
/// produce histogram buckets and are incompatible with aggregate metrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FacetField {
    Make,
    Model,
    InventoryType,
    Year,
    Trim,
    Version,
    BaseExteriorColor,
    ExteriorColor,
    BaseInteriorColor,
    InteriorColor,
    SeatingCapacity,
    Doors,
    Engine,
    State,
    Drivetrain,
    AssemblyLocation,
    AssemblyCountry,
    Transmission,
    FuelType,
    BodyType,
    Cylinders,
    DealerType,
    AvailabilityStatus,
    OptionsPackages,
    Features,
    Keywords,
    Price,
    Msrp,
    Miles,
    DaysOnMarket,
}

impl FacetField {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Make => "make",
            Self::Model => "model",
            Self::InventoryType => "inventory_type",
            Self::Year => "year",
            Self::Trim => "trim",
            Self::Version => "version",
            Self::BaseExteriorColor => "base_exterior_color",
            Self::ExteriorColor => "exterior_color",
            Self::BaseInteriorColor => "base_interior_color",
            Self::InteriorColor => "interior_color",
            Self::SeatingCapacity => "seating_capacity",
            Self::Doors => "doors",
            Self::Engine => "engine",
            Self::State => "state",
            Self::Drivetrain => "drivetrain",
            Self::AssemblyLocation => "assembly_location",
            Self::AssemblyCountry => "assembly_country",
            Self::Transmission => "transmission",
            Self::FuelType => "fuel_type",
            Self::BodyType => "body_type",
            Self::Cylinders => "cylinders",
            Self::DealerType => "dealer_type",
            Self::AvailabilityStatus => "availability_status",
            Self::OptionsPackages => "options_packages",
            Self::Features => "features",
            Self::Keywords => "keywords",
            Self::Price => "price",
            Self::Msrp => "msrp",
            Self::Miles => "miles",
            Self::DaysOnMarket => "days_on_market",
        }
    }

    pub fn is_categorical(&self) -> bool {
        !matches!(
            self,
            Self::Price | Self::Msrp | Self::Miles | Self::DaysOnMarket
        )
    }

    pub fn is_numeric_range(&self) -> bool {
        !self.is_categorical()
    }
}

// ── FacetMetric ───────────────────────────────────────────────────────────────

/// Measure used in an aggregate metric calculation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FacetMetricMeasure {
    Price,
    Miles,
    Msrp,
    DaysOnMarket,
    DiscountFromMsrp,
}

impl FacetMetricMeasure {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Price => "price",
            Self::Miles => "miles",
            Self::Msrp => "msrp",
            Self::DaysOnMarket => "days_on_market",
            Self::DiscountFromMsrp => "discount_from_msrp",
        }
    }
}

/// Aggregate function applied to a measure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FacetMetricAggregate {
    Mean,
    P5,
    P25,
    Median,
    P75,
    P95,
}

impl FacetMetricAggregate {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mean => "mean",
            Self::P5 => "p5",
            Self::P25 => "p25",
            Self::Median => "median",
            Self::P75 => "p75",
            Self::P95 => "p95",
        }
    }
}

/// The `metric` request parameter for `/v1/facets`.
///
/// `Count` emits `metric=count`. `Aggregate` emits `metric={measure}.{aggregate}`.
/// Omitting `metric` (i.e. `None` on `FacetsFilter`) lets the API default to count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FacetMetric {
    Count,
    Aggregate {
        measure: FacetMetricMeasure,
        aggregate: FacetMetricAggregate,
    },
}

impl FacetMetric {
    pub fn as_str(&self) -> String {
        match self {
            Self::Count => "count".to_string(),
            Self::Aggregate { measure, aggregate } => {
                format!("{}.{}", measure.as_str(), aggregate.as_str())
            }
        }
    }
}

// ── FacetSort ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub enum FacetSort {
    Count,
    #[default]
    CountDesc,
    Metric,
    MetricDesc,
}

impl FacetSort {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::CountDesc => "-count",
            Self::Metric => "metric",
            Self::MetricDesc => "-metric",
        }
    }
}

// ── FacetsFilter ──────────────────────────────────────────────────────────────

pub struct FacetsFilter {
    pub base: ListingsFilterBase,
    pub facets: Vec<FacetField>,
    pub facet_value_limit: Option<u32>,
    pub metric: Option<FacetMetric>,
    pub sort: FacetSort,
}

impl FacetsFilter {
    /// Construct with required facets; all other fields use defaults.
    pub fn new(facets: Vec<FacetField>) -> Self {
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

    /// Validate filter constraints before sending a request.
    pub fn validate(&self) -> Result<(), VisorError> {
        if self.facets.is_empty() {
            return Err(VisorError::InvalidFilter {
                message: "facets must not be empty".to_string(),
            });
        }

        if let Some(limit) = self.facet_value_limit {
            if limit > 100 {
                return Err(VisorError::InvalidFilter {
                    message: format!("facet_value_limit must be <= 100, got {limit}"),
                });
            }
        }

        if let Some(FacetMetric::Aggregate { .. }) = &self.metric {
            if self.facets.len() != 1 {
                return Err(VisorError::InvalidFilter {
                    message: format!(
                        "aggregate metric requires exactly one facet, got {}",
                        self.facets.len()
                    ),
                });
            }
            if !self.facets[0].is_categorical() {
                return Err(VisorError::InvalidFilter {
                    message: format!(
                        "aggregate metric requires a categorical facet; '{}' is a numeric range facet",
                        self.facets[0].as_str()
                    ),
                });
            }
        }

        match self.sort {
            FacetSort::Metric | FacetSort::MetricDesc
                if !matches!(self.metric, Some(FacetMetric::Aggregate { .. })) =>
            {
                return Err(VisorError::InvalidFilter {
                    message: "metric sort requires an aggregate metric".to_string(),
                });
            }
            _ => {}
        }

        Ok(())
    }
}

// ── Response models ───────────────────────────────────────────────────────────

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
