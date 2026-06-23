use uuid::Uuid;

use crate::models::dealers::DealerType;
use crate::models::filter_types::{
    AvailabilityStatus, CountryCode, GeoFilter, HistoryKeyword, InventoryModeFilter, InventoryType,
    StateCode, VinPattern,
};

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

impl ListingInclude {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PriceHistory => "price_history",
            Self::Options => "options",
        }
    }
}

/// Shared filter fields used by both `ListingsFilter` and `FacetsFilter`.
#[derive(Debug, Clone, Default)]
pub struct ListingsFilterBase {
    // ── Vehicle ──────────────────────────────────────────────────────────────
    pub make: Option<Vec<String>>,
    pub model: Option<Vec<String>>,
    pub trim: Option<Vec<String>>,
    pub year: Option<Vec<u16>>,
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
    pub seating_capacity: Option<Vec<u8>>,
    pub cylinders: Option<Vec<u8>>,
    pub doors: Option<Vec<u8>>,
    pub options_packages: Option<Vec<String>>,
    pub features: Option<Vec<String>>,
    /// Pipe-separated on the wire.
    pub assembly_location: Option<Vec<String>>,
    pub assembly_country: Option<Vec<CountryCode>>,
    pub vin_pattern: Option<Vec<VinPattern>>,
    pub keywords: Option<Vec<HistoryKeyword>>,

    // ── Dealer ───────────────────────────────────────────────────────────────
    pub state: Option<Vec<StateCode>>,
    pub dealer_id: Option<Vec<Uuid>>,
    pub dealer_type: Option<Vec<DealerType>>,

    // ── Inventory status ─────────────────────────────────────────────────────
    pub availability_status: Option<Vec<AvailabilityStatus>>,
    pub inventory_type: Option<Vec<InventoryType>>,
    /// Controls which inventory mode is queried and its associated parameters.
    /// Defaults to `Active`. Replaces the old `inventory_status`,
    /// `sold_within_days`, and `snapshot_date` separate fields.
    pub inventory_mode: InventoryModeFilter,

    // ── Numeric ranges ───────────────────────────────────────────────────────
    pub min_price: Option<u32>,
    pub max_price: Option<u32>,
    pub min_mileage: Option<u32>,
    pub max_mileage: Option<u32>,
    pub min_msrp: Option<u32>,
    pub max_msrp: Option<u32>,
    pub min_days_on_market: Option<u32>,
    pub max_days_on_market: Option<u32>,

    // ── Geographic ───────────────────────────────────────────────────────────
    /// Radius or bounding-box constraint. Radius/bbox are mutually exclusive;
    /// the `GeoFilter` enum makes invalid combinations impossible.
    pub geo: Option<GeoFilter>,

    // ── Exclusions ───────────────────────────────────────────────────────────
    pub exclude_make: Option<Vec<String>>,
    pub exclude_model: Option<Vec<String>>,
    pub exclude_trim: Option<Vec<String>>,
    pub exclude_year: Option<Vec<u16>>,
    pub exclude_state: Option<Vec<StateCode>>,
    pub exclude_inventory_type: Option<Vec<InventoryType>>,
    pub exclude_body_type: Option<Vec<String>>,
    pub exclude_transmission: Option<Vec<String>>,
    pub exclude_drivetrain: Option<Vec<String>>,
    pub exclude_version: Option<Vec<String>>,
    pub exclude_engine: Option<Vec<String>>,
    /// Plus-separated on the wire.
    pub exclude_assembly_location: Option<Vec<String>>,
    pub exclude_assembly_country: Option<Vec<CountryCode>>,
    pub exclude_exterior_color: Option<Vec<String>>,
    pub exclude_interior_color: Option<Vec<String>>,
    pub exclude_base_exterior_color: Option<Vec<String>>,
    pub exclude_base_interior_color: Option<Vec<String>>,
    pub exclude_options_packages: Option<Vec<String>>,
    pub exclude_features: Option<Vec<String>>,
    pub exclude_fuel_type: Option<Vec<String>>,
    pub exclude_powertrain_type: Option<Vec<String>>,
    pub exclude_keywords: Option<Vec<HistoryKeyword>>,
}
