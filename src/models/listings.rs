use std::collections::HashMap;

use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::Value;

use crate::error::VisorError;
use crate::models::base::{ListingInclude, ListingsFilterBase, SortOrder};
use crate::models::common::{
    DealerRef, Pagination, PriceHistoryEntry, VehicleOption, VehicleRecord,
};
use crate::models::filter_types::ListingField;

pub struct ListingsFilter {
    pub base: ListingsFilterBase,
    pub limit: u32,
    pub offset: u32,
    pub sort: SortOrder,
    pub fields: Option<Vec<ListingField>>,
    pub include: Option<Vec<ListingInclude>>,
}

impl Default for ListingsFilter {
    fn default() -> Self {
        Self {
            base: ListingsFilterBase::default(),
            limit: 50,
            offset: 0,
            sort: SortOrder::DaysOnMarket,
            fields: None,
            include: None,
        }
    }
}

impl ListingsFilter {
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
pub struct ListingSummary {
    pub id: String,
    pub vin: String,
    pub year: Option<i32>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub trim: Option<String>,
    pub version: Option<String>,
    pub body_type: Option<String>,
    pub drivetrain: Option<String>,
    pub fuel_type: Option<String>,
    pub powertrain_type: Option<String>,
    pub transmission: Option<String>,
    pub engine: Option<String>,
    pub cylinders: Option<i32>,
    pub doors: Option<i32>,
    pub seating_capacity: Option<i32>,
    pub exterior_color: Option<String>,
    pub interior_color: Option<String>,
    pub base_exterior_color: Option<String>,
    pub base_interior_color: Option<String>,
    pub msrp: Option<i32>,
    pub discount_from_msrp: Option<i32>,
    pub price: Option<i32>,
    pub miles: Option<i32>,
    pub days_on_market: Option<i32>,
    pub status: Option<String>,
    pub inventory_status: Option<String>,
    pub inventory_type: Option<String>,
    pub stock_number: Option<String>,
    pub vdp_url: Option<String>,
    pub sold_date: Option<NaiveDate>,
    pub dealer_id: Option<String>,
    pub dealer_name: Option<String>,
    pub dealer_type: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub distance_miles: Option<f64>,
    #[serde(default)]
    pub photo_urls: Vec<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub options_packages: Vec<String>,
    #[serde(default)]
    pub price_history: Vec<PriceHistoryEntry>,
    #[serde(default)]
    pub options: Vec<VehicleOption>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListingDetail {
    pub id: String,
    pub vin: String,
    pub status: String,
    pub price: Option<i32>,
    pub miles: Option<i32>,
    pub inventory_type: String,
    pub stock_number: Option<String>,
    pub vdp_url: Option<String>,
    pub vhr_url: Option<String>,
    #[serde(default)]
    pub photo_urls: Vec<String>,
    pub photo_url_primary: Option<String>,
    pub inventory_date: Option<NaiveDate>,
    pub sold_date: Option<NaiveDate>,
    pub last_checked_at: Option<String>,
    pub dealer: DealerRef,
    pub vehicle: VehicleRecord,
    #[serde(default)]
    pub price_history: Option<Vec<PriceHistoryEntry>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListingSnapshot {
    pub id: String,
    pub price: Option<i32>,
    pub miles: Option<i32>,
    pub inventory_type: String,
    pub stock_number: Option<String>,
    pub vdp_url: Option<String>,
    pub vhr_url: Option<String>,
    #[serde(default)]
    pub photo_urls: Vec<String>,
    pub photo_url_primary: Option<String>,
    pub inventory_date: Option<NaiveDate>,
    pub sold_date: Option<NaiveDate>,
    pub last_checked_at: Option<String>,
    pub dealer: DealerRef,
    #[serde(default)]
    pub price_history: Option<Vec<PriceHistoryEntry>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListingsPage {
    pub data: Vec<ListingSummary>,
    pub pagination: Pagination,
    #[serde(default)]
    pub meta: HashMap<String, Value>,
}
