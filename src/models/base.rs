use chrono::NaiveDate;

use crate::models::common::BBox;

#[derive(Debug, Clone, Default)]
pub enum InventoryStatus {
    #[default]
    Active,
    Sold,
}

#[derive(Debug, Clone, Default)]
pub enum SortOrder {
    #[default]
    DaysOnMarket,
    DaysOnMarketDesc,
    Price,
    PriceDesc,
    Miles,
    MilesDesc,
    Msrp,
    MsrpDesc,
    Discount,
    DiscountDesc,
    Distance,
}

#[derive(Debug, Clone)]
pub enum ListingInclude {
    PriceHistory,
    Options,
}

/// Shared filter fields used by both ListingsFilter and FacetsFilter.
#[derive(Debug, Clone, Default)]
pub struct ListingsFilterBase {
    pub make: Option<Vec<String>>,
    pub model: Option<Vec<String>>,
    pub trim: Option<Vec<String>>,
    pub year: Option<Vec<i32>>,
    pub state: Option<Vec<String>>,
    pub dealer_id: Option<Vec<String>>,
    pub dealer_type: Option<Vec<String>>,
    pub availability_status: Option<Vec<String>>,
    pub inventory_type: Option<Vec<String>>,
    pub body_type: Option<Vec<String>>,
    pub transmission: Option<Vec<String>>,
    pub drivetrain: Option<Vec<String>>,
    pub fuel_type: Option<Vec<String>>,
    pub powertrain_type: Option<Vec<String>>,
    pub engine: Option<Vec<String>>,
    pub version: Option<Vec<String>>,
    pub exterior_color: Option<Vec<String>>,
    pub interior_color: Option<Vec<String>>,
    pub base_exterior_color: Option<Vec<String>>,
    pub base_interior_color: Option<Vec<String>>,
    pub seating_capacity: Option<Vec<i32>>,
    pub cylinders: Option<Vec<i32>>,
    pub doors: Option<Vec<i32>>,
    pub options_packages: Option<Vec<String>>,
    pub features: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    pub vin_pattern: Option<Vec<String>>,
    /// Pipe-separated on the wire.
    pub assembly_location: Option<Vec<String>>,
    pub assembly_country: Option<Vec<String>>,
    pub exclude_make: Option<Vec<String>>,
    pub exclude_model: Option<Vec<String>>,
    pub exclude_trim: Option<Vec<String>>,
    pub exclude_year: Option<Vec<i32>>,
    pub exclude_state: Option<Vec<String>>,
    pub exclude_inventory_type: Option<Vec<String>>,
    pub exclude_body_type: Option<Vec<String>>,
    pub exclude_transmission: Option<Vec<String>>,
    pub exclude_drivetrain: Option<Vec<String>>,
    pub exclude_version: Option<Vec<String>>,
    pub exclude_engine: Option<Vec<String>>,
    /// Plus-separated on the wire.
    pub exclude_assembly_location: Option<Vec<String>>,
    pub exclude_assembly_country: Option<Vec<String>>,
    pub exclude_exterior_color: Option<Vec<String>>,
    pub exclude_interior_color: Option<Vec<String>>,
    pub exclude_base_exterior_color: Option<Vec<String>>,
    pub exclude_base_interior_color: Option<Vec<String>>,
    pub exclude_options_packages: Option<Vec<String>>,
    pub exclude_features: Option<Vec<String>>,
    pub exclude_fuel_type: Option<Vec<String>>,
    pub exclude_powertrain_type: Option<Vec<String>>,
    pub exclude_keywords: Option<Vec<String>>,
    pub min_price: Option<i32>,
    pub max_price: Option<i32>,
    pub min_mileage: Option<i32>,
    pub max_mileage: Option<i32>,
    pub min_msrp: Option<i32>,
    pub max_msrp: Option<i32>,
    pub min_days_on_market: Option<i32>,
    pub max_days_on_market: Option<i32>,
    pub inventory_status: InventoryStatus,
    pub sold_within_days: Option<i32>,
    pub snapshot_date: Option<NaiveDate>,
    pub postal_code: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub radius: Option<f64>,
    pub bbox: Option<BBox>,
}
