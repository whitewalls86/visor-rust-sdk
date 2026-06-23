use uuid::Uuid;

use crate::error::VisorError;
use crate::models::dealers::DealerType;
use crate::models::filter_types::{
    AvailabilityStatus, GeoFilter, GeoOrigin, HistoryKeyword, InventoryModeFilter, InventoryType,
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

impl SortOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DaysOnMarket => "days_on_market",
            Self::DaysOnMarketDesc => "-days_on_market",
            Self::Price => "price",
            Self::PriceDesc => "-price",
            Self::Miles => "miles",
            Self::MilesDesc => "-miles",
            Self::Msrp => "msrp",
            Self::MsrpDesc => "-msrp",
            Self::Discount => "discount",
            Self::DiscountDesc => "-discount",
            Self::Distance => "distance",
        }
    }
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
    pub assembly_country: Option<Vec<String>>,
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
    pub exclude_assembly_country: Option<Vec<String>>,
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

impl ListingsFilterBase {
    /// Append all base-filter wire params to `params`.
    ///
    /// Called by `ListingsFilter::to_params()` and `FacetsFilter::to_params()`
    /// to avoid duplicating field-by-field serialization logic.
    pub(crate) fn append_params(&self, params: &mut Vec<(String, String)>) {
        // ── Vehicle ──────────────────────────────────────────────────────────
        push_str_list(params, "make", self.make.as_deref(), ",");
        push_str_list(params, "model", self.model.as_deref(), ",");
        push_str_list(params, "trim", self.trim.as_deref(), ",");
        push_num_list(params, "year", self.year.as_deref());
        push_str_list(params, "body_type", self.body_type.as_deref(), ",");
        push_str_list(params, "transmission", self.transmission.as_deref(), ",");
        push_str_list(params, "drivetrain", self.drivetrain.as_deref(), ",");
        push_str_list(params, "fuel_type", self.fuel_type.as_deref(), ",");
        push_str_list(
            params,
            "powertrain_type",
            self.powertrain_type.as_deref(),
            ",",
        );
        push_str_list(params, "engine", self.engine.as_deref(), ",");
        push_str_list(params, "version", self.version.as_deref(), ",");
        push_str_list(
            params,
            "exterior_color",
            self.exterior_color.as_deref(),
            ",",
        );
        push_str_list(
            params,
            "interior_color",
            self.interior_color.as_deref(),
            ",",
        );
        push_str_list(
            params,
            "base_exterior_color",
            self.base_exterior_color.as_deref(),
            ",",
        );
        push_str_list(
            params,
            "base_interior_color",
            self.base_interior_color.as_deref(),
            ",",
        );
        push_u8_list(params, "seating_capacity", self.seating_capacity.as_deref());
        push_u8_list(params, "cylinders", self.cylinders.as_deref());
        push_u8_list(params, "doors", self.doors.as_deref());
        push_str_list(
            params,
            "options_packages",
            self.options_packages.as_deref(),
            ",",
        );
        push_str_list(params, "features", self.features.as_deref(), ",");
        push_str_list(
            params,
            "assembly_location",
            self.assembly_location.as_deref(),
            "|",
        );
        push_str_list(
            params,
            "assembly_country",
            self.assembly_country.as_deref(),
            ",",
        );
        if let Some(pats) = &self.vin_pattern {
            if !pats.is_empty() {
                let joined = pats
                    .iter()
                    .map(|p| p.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                params.push(("vin_pattern".to_string(), joined));
            }
        }
        if let Some(kws) = &self.keywords {
            if !kws.is_empty() {
                let joined = kws.iter().map(|k| k.as_str()).collect::<Vec<_>>().join(",");
                params.push(("keywords".to_string(), joined));
            }
        }

        // ── Dealer ───────────────────────────────────────────────────────────
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
        if let Some(dt) = &self.dealer_type {
            if !dt.is_empty() {
                let joined = dt.iter().map(|d| d.as_str()).collect::<Vec<_>>().join(",");
                params.push(("dealer_type".to_string(), joined));
            }
        }

        // ── Inventory status ─────────────────────────────────────────────────
        if let Some(avail) = &self.availability_status {
            if !avail.is_empty() {
                let joined = avail
                    .iter()
                    .map(|a| a.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                params.push(("availability_status".to_string(), joined));
            }
        }
        if let Some(inv_type) = &self.inventory_type {
            if !inv_type.is_empty() {
                let joined = inv_type
                    .iter()
                    .map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                params.push(("inventory_type".to_string(), joined));
            }
        }
        match &self.inventory_mode {
            InventoryModeFilter::Active => {}
            InventoryModeFilter::Sold { sold_within_days } => {
                params.push(("inventory_status".to_string(), "sold".to_string()));
                if let Some(days) = sold_within_days {
                    params.push(("sold_within_days".to_string(), days.to_string()));
                }
            }
            InventoryModeFilter::Snapshot { date } => {
                params.push(("snapshot_date".to_string(), date.to_string()));
            }
        }

        // ── Numeric ranges ───────────────────────────────────────────────────
        push_u32_opt(params, "min_price", self.min_price);
        push_u32_opt(params, "max_price", self.max_price);
        push_u32_opt(params, "min_mileage", self.min_mileage);
        push_u32_opt(params, "max_mileage", self.max_mileage);
        push_u32_opt(params, "min_msrp", self.min_msrp);
        push_u32_opt(params, "max_msrp", self.max_msrp);
        push_u32_opt(params, "min_days_on_market", self.min_days_on_market);
        push_u32_opt(params, "max_days_on_market", self.max_days_on_market);

        // ── Geographic ───────────────────────────────────────────────────────
        if let Some(geo) = &self.geo {
            append_geo(params, geo);
        }

        // ── Exclusions ───────────────────────────────────────────────────────
        push_str_list(params, "exclude_make", self.exclude_make.as_deref(), ",");
        push_str_list(params, "exclude_model", self.exclude_model.as_deref(), ",");
        push_str_list(params, "exclude_trim", self.exclude_trim.as_deref(), ",");
        push_num_list(params, "exclude_year", self.exclude_year.as_deref());
        if let Some(states) = &self.exclude_state {
            if !states.is_empty() {
                let joined = states
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                params.push(("exclude_state".to_string(), joined));
            }
        }
        if let Some(inv_type) = &self.exclude_inventory_type {
            if !inv_type.is_empty() {
                let joined = inv_type
                    .iter()
                    .map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                params.push(("exclude_inventory_type".to_string(), joined));
            }
        }
        push_str_list(
            params,
            "exclude_body_type",
            self.exclude_body_type.as_deref(),
            ",",
        );
        push_str_list(
            params,
            "exclude_transmission",
            self.exclude_transmission.as_deref(),
            ",",
        );
        push_str_list(
            params,
            "exclude_drivetrain",
            self.exclude_drivetrain.as_deref(),
            ",",
        );
        push_str_list(
            params,
            "exclude_version",
            self.exclude_version.as_deref(),
            ",",
        );
        push_str_list(
            params,
            "exclude_engine",
            self.exclude_engine.as_deref(),
            ",",
        );
        push_str_list(
            params,
            "exclude_assembly_location",
            self.exclude_assembly_location.as_deref(),
            "+",
        );
        push_str_list(
            params,
            "exclude_assembly_country",
            self.exclude_assembly_country.as_deref(),
            ",",
        );
        push_str_list(
            params,
            "exclude_exterior_color",
            self.exclude_exterior_color.as_deref(),
            ",",
        );
        push_str_list(
            params,
            "exclude_interior_color",
            self.exclude_interior_color.as_deref(),
            ",",
        );
        push_str_list(
            params,
            "exclude_base_exterior_color",
            self.exclude_base_exterior_color.as_deref(),
            ",",
        );
        push_str_list(
            params,
            "exclude_base_interior_color",
            self.exclude_base_interior_color.as_deref(),
            ",",
        );
        push_str_list(
            params,
            "exclude_options_packages",
            self.exclude_options_packages.as_deref(),
            ",",
        );
        push_str_list(
            params,
            "exclude_features",
            self.exclude_features.as_deref(),
            ",",
        );
        push_str_list(
            params,
            "exclude_fuel_type",
            self.exclude_fuel_type.as_deref(),
            ",",
        );
        push_str_list(
            params,
            "exclude_powertrain_type",
            self.exclude_powertrain_type.as_deref(),
            ",",
        );
        if let Some(kws) = &self.exclude_keywords {
            if !kws.is_empty() {
                let joined = kws.iter().map(|k| k.as_str()).collect::<Vec<_>>().join(",");
                params.push(("exclude_keywords".to_string(), joined));
            }
        }
    }

    /// Validate all shared filter constraints.
    ///
    /// Called by `ListingsFilter::validate()` and `FacetsFilter::validate()`.
    pub fn validate(&self) -> Result<(), VisorError> {
        if let Some(ids) = &self.dealer_id {
            if ids.len() > 50 {
                return Err(VisorError::InvalidFilter {
                    message: format!("dealer_id accepts at most 50 IDs, got {}", ids.len()),
                });
            }
        }
        if let Some(pats) = &self.vin_pattern {
            if pats.len() > 10 {
                return Err(VisorError::InvalidFilter {
                    message: format!(
                        "vin_pattern accepts at most 10 patterns, got {}",
                        pats.len()
                    ),
                });
            }
        }

        check_range_pair(self.min_price, self.max_price, "price")?;
        check_range_pair(self.min_mileage, self.max_mileage, "mileage")?;
        check_range_pair(self.min_msrp, self.max_msrp, "msrp")?;
        check_range_pair(
            self.min_days_on_market,
            self.max_days_on_market,
            "days_on_market",
        )?;

        if let Some(caps) = &self.seating_capacity {
            for &cap in caps {
                if cap == 0 {
                    return Err(VisorError::InvalidFilter {
                        message: "seating_capacity values must be positive (> 0)".to_string(),
                    });
                }
            }
        }
        if let Some(doors) = &self.doors {
            for &d in doors {
                if d == 0 {
                    return Err(VisorError::InvalidFilter {
                        message: "doors values must be positive (> 0)".to_string(),
                    });
                }
            }
        }

        if let InventoryModeFilter::Sold {
            sold_within_days: Some(days),
        } = &self.inventory_mode
        {
            if *days == 0 {
                return Err(VisorError::InvalidFilter {
                    message: "sold_within_days must be positive (> 0)".to_string(),
                });
            }
        }

        validate_str_vec_field("make", self.make.as_deref())?;
        validate_str_vec_field("model", self.model.as_deref())?;
        validate_str_vec_field("trim", self.trim.as_deref())?;
        validate_str_vec_field("body_type", self.body_type.as_deref())?;
        validate_str_vec_field("transmission", self.transmission.as_deref())?;
        validate_str_vec_field("drivetrain", self.drivetrain.as_deref())?;
        validate_str_vec_field("fuel_type", self.fuel_type.as_deref())?;
        validate_str_vec_field("powertrain_type", self.powertrain_type.as_deref())?;
        validate_str_vec_field("engine", self.engine.as_deref())?;
        validate_str_vec_field("version", self.version.as_deref())?;
        validate_str_vec_field("exterior_color", self.exterior_color.as_deref())?;
        validate_str_vec_field("interior_color", self.interior_color.as_deref())?;
        validate_str_vec_field("base_exterior_color", self.base_exterior_color.as_deref())?;
        validate_str_vec_field("base_interior_color", self.base_interior_color.as_deref())?;
        validate_str_vec_field("options_packages", self.options_packages.as_deref())?;
        validate_str_vec_field("features", self.features.as_deref())?;
        validate_str_vec_field("assembly_location", self.assembly_location.as_deref())?;
        validate_str_vec_field("assembly_country", self.assembly_country.as_deref())?;
        validate_str_vec_field("exclude_make", self.exclude_make.as_deref())?;
        validate_str_vec_field("exclude_model", self.exclude_model.as_deref())?;
        validate_str_vec_field("exclude_trim", self.exclude_trim.as_deref())?;
        validate_str_vec_field("exclude_body_type", self.exclude_body_type.as_deref())?;
        validate_str_vec_field("exclude_transmission", self.exclude_transmission.as_deref())?;
        validate_str_vec_field("exclude_drivetrain", self.exclude_drivetrain.as_deref())?;
        validate_str_vec_field("exclude_version", self.exclude_version.as_deref())?;
        validate_str_vec_field("exclude_engine", self.exclude_engine.as_deref())?;
        validate_str_vec_field(
            "exclude_assembly_location",
            self.exclude_assembly_location.as_deref(),
        )?;
        validate_str_vec_field(
            "exclude_assembly_country",
            self.exclude_assembly_country.as_deref(),
        )?;
        validate_str_vec_field(
            "exclude_exterior_color",
            self.exclude_exterior_color.as_deref(),
        )?;
        validate_str_vec_field(
            "exclude_interior_color",
            self.exclude_interior_color.as_deref(),
        )?;
        validate_str_vec_field(
            "exclude_base_exterior_color",
            self.exclude_base_exterior_color.as_deref(),
        )?;
        validate_str_vec_field(
            "exclude_base_interior_color",
            self.exclude_base_interior_color.as_deref(),
        )?;
        validate_str_vec_field(
            "exclude_options_packages",
            self.exclude_options_packages.as_deref(),
        )?;
        validate_str_vec_field("exclude_features", self.exclude_features.as_deref())?;
        validate_str_vec_field("exclude_fuel_type", self.exclude_fuel_type.as_deref())?;
        validate_str_vec_field(
            "exclude_powertrain_type",
            self.exclude_powertrain_type.as_deref(),
        )?;

        Ok(())
    }
}

// ── Private serialization helpers ─────────────────────────────────────────────

fn push_str_list(
    params: &mut Vec<(String, String)>,
    key: &str,
    values: Option<&[String]>,
    sep: &str,
) {
    if let Some(vs) = values {
        if !vs.is_empty() {
            params.push((key.to_string(), vs.join(sep)));
        }
    }
}

fn push_u8_list(params: &mut Vec<(String, String)>, key: &str, values: Option<&[u8]>) {
    if let Some(vs) = values {
        if !vs.is_empty() {
            let joined = vs
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",");
            params.push((key.to_string(), joined));
        }
    }
}

fn push_num_list<T: ToString>(params: &mut Vec<(String, String)>, key: &str, values: Option<&[T]>) {
    if let Some(vs) = values {
        if !vs.is_empty() {
            let joined = vs
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",");
            params.push((key.to_string(), joined));
        }
    }
}

fn push_u32_opt(params: &mut Vec<(String, String)>, key: &str, value: Option<u32>) {
    if let Some(v) = value {
        params.push((key.to_string(), v.to_string()));
    }
}

fn append_geo(params: &mut Vec<(String, String)>, geo: &GeoFilter) {
    match geo {
        GeoFilter::Origin(origin) => append_geo_origin(params, origin),
        GeoFilter::Radius { origin, miles } => {
            append_geo_origin(params, origin);
            params.push(("radius".to_string(), format!("{}", miles.value())));
        }
        GeoFilter::BBox(bbox) => {
            params.push((
                "bbox".to_string(),
                format!(
                    "{},{},{},{}",
                    bbox.west(),
                    bbox.south(),
                    bbox.east(),
                    bbox.north()
                ),
            ));
        }
    }
}

fn append_geo_origin(params: &mut Vec<(String, String)>, origin: &GeoOrigin) {
    match origin {
        GeoOrigin::PostalCode(pc) => {
            params.push(("postal_code".to_string(), pc.as_str().to_string()));
        }
        GeoOrigin::Coordinates {
            latitude,
            longitude,
        } => {
            params.push(("latitude".to_string(), format!("{}", latitude.value())));
            params.push(("longitude".to_string(), format!("{}", longitude.value())));
        }
    }
}

// ── Private validation helpers ────────────────────────────────────────────────

fn check_range_pair(min: Option<u32>, max: Option<u32>, name: &str) -> Result<(), VisorError> {
    if let (Some(mn), Some(mx)) = (min, max) {
        if mn > mx {
            return Err(VisorError::InvalidFilter {
                message: format!("{name}: min ({mn}) must be <= max ({mx})"),
            });
        }
    }
    Ok(())
}

fn validate_str_vec_field(name: &str, values: Option<&[String]>) -> Result<(), VisorError> {
    if let Some(vs) = values {
        for v in vs {
            if v.trim().is_empty() {
                return Err(VisorError::InvalidFilter {
                    message: format!("{name}: empty or whitespace-only elements are not allowed"),
                });
            }
        }
    }
    Ok(())
}
