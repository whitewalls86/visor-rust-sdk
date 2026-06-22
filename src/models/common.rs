use chrono::NaiveDate;
use serde::Deserialize;

/// Bounding box for map-viewport filtering: west, south, east, north.
#[derive(Debug, Clone)]
pub struct BBox {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct VehicleOption {
    pub code: String,
    pub name: String,
    pub msrp: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct VehicleBuild {
    pub year: i32,
    pub make: String,
    pub model: String,
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
    pub assembly_location: Option<String>,
    pub assembly_country: Option<String>,
    #[serde(default)]
    pub window_sticker_verified: bool,
    pub base_msrp: Option<i32>,
    pub combined_msrp: Option<i32>,
    pub options: Option<Vec<VehicleOption>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VehicleRecord {
    pub vin: String,
    pub status: String,
    pub build: VehicleBuild,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceHistoryEntry {
    pub date: NaiveDate,
    pub price: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DealerRef {
    pub dealer_id: String,
    pub name: String,
    pub city: String,
    pub state: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    pub limit: i32,
    pub offset: i32,
    pub total: i32,
    pub next_offset: Option<i32>,
}
