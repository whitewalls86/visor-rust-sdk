use serde::Deserialize;

use crate::models::common::VehicleBuild;
use crate::models::listings::ListingSnapshot;

#[derive(Debug, Clone, Deserialize)]
pub struct VinDetail {
    pub vin: String,
    pub status: String,
    pub build: VehicleBuild,
    pub latest_listing: Option<ListingSnapshot>,
}
