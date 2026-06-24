use serde::Deserialize;

use crate::error::VisorError;
use crate::models::common::VehicleBuild;
use crate::models::listings::ListingSnapshot;

/// A validated 17-character VIN.
///
/// Accepts upper- or lower-case input, normalizes to uppercase.
/// I, O, and Q are rejected per the ISO 3779 standard.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Vin(String);

impl Vin {
    pub fn new(raw: impl AsRef<str>) -> Result<Self, VisorError> {
        let vin = raw.as_ref().to_ascii_uppercase();
        if vin.len() != 17 {
            return Err(VisorError::InvalidFilter {
                message: format!("VIN must be 17 characters, got {}", vin.len()),
            });
        }
        for (i, ch) in vin.chars().enumerate() {
            if !ch.is_ascii_alphanumeric() || matches!(ch, 'I' | 'O' | 'Q') {
                return Err(VisorError::InvalidFilter {
                    message: format!("invalid VIN character '{ch}' at position {i}"),
                });
            }
        }
        Ok(Self(vin))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct VinDetail {
    pub vin: String,
    pub status: String,
    pub build: VehicleBuild,
    pub latest_listing: Option<ListingSnapshot>,
}
