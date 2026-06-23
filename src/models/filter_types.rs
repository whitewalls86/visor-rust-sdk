use chrono::NaiveDate;

use crate::error::VisorError;
use crate::models::common::BBox;

// ── Closed-vocabulary enums ────────────────────────────────────────────────

/// Projection fields for the filter-listings `fields` parameter.
///
/// The API rejects unknown field names; use this enum to stay in sync.
/// `Default` includes the standard fieldset; any other variant appends to it.
#[derive(Debug, Clone)]
pub enum ListingField {
    Default,
    Id,
    Vin,
    Year,
    Make,
    Model,
    Trim,
    Version,
    BodyType,
    Drivetrain,
    FuelType,
    PowertrainType,
    Transmission,
    Engine,
    Cylinders,
    Doors,
    SeatingCapacity,
    ExteriorColor,
    InteriorColor,
    BaseExteriorColor,
    BaseInteriorColor,
    Msrp,
    DiscountFromMsrp,
    Price,
    Miles,
    DaysOnMarket,
    Status,
    InventoryStatus,
    InventoryType,
    StockNumber,
    VdpUrl,
    SoldDate,
    DealerId,
    DealerName,
    DealerType,
    City,
    State,
    Latitude,
    Longitude,
    DistanceMiles,
    PhotoUrls,
    Features,
    OptionsPackages,
}

impl ListingField {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Id => "id",
            Self::Vin => "vin",
            Self::Year => "year",
            Self::Make => "make",
            Self::Model => "model",
            Self::Trim => "trim",
            Self::Version => "version",
            Self::BodyType => "body_type",
            Self::Drivetrain => "drivetrain",
            Self::FuelType => "fuel_type",
            Self::PowertrainType => "powertrain_type",
            Self::Transmission => "transmission",
            Self::Engine => "engine",
            Self::Cylinders => "cylinders",
            Self::Doors => "doors",
            Self::SeatingCapacity => "seating_capacity",
            Self::ExteriorColor => "exterior_color",
            Self::InteriorColor => "interior_color",
            Self::BaseExteriorColor => "base_exterior_color",
            Self::BaseInteriorColor => "base_interior_color",
            Self::Msrp => "msrp",
            Self::DiscountFromMsrp => "discount_from_msrp",
            Self::Price => "price",
            Self::Miles => "miles",
            Self::DaysOnMarket => "days_on_market",
            Self::Status => "status",
            Self::InventoryStatus => "inventory_status",
            Self::InventoryType => "inventory_type",
            Self::StockNumber => "stock_number",
            Self::VdpUrl => "vdp_url",
            Self::SoldDate => "sold_date",
            Self::DealerId => "dealer_id",
            Self::DealerName => "dealer_name",
            Self::DealerType => "dealer_type",
            Self::City => "city",
            Self::State => "state",
            Self::Latitude => "latitude",
            Self::Longitude => "longitude",
            Self::DistanceMiles => "distance_miles",
            Self::PhotoUrls => "photo_urls",
            Self::Features => "features",
            Self::OptionsPackages => "options_packages",
        }
    }
}

/// Availability status filter. Closed vocabulary.
#[derive(Debug, Clone)]
pub enum AvailabilityStatus {
    Stock,
    Transit,
    Build,
}

impl AvailabilityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stock => "stock",
            Self::Transit => "transit",
            Self::Build => "build",
        }
    }
}

/// Inventory type filter. Closed vocabulary.
#[derive(Debug, Clone)]
pub enum InventoryType {
    New,
    Used,
    Certified,
}

impl InventoryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Used => "used",
            Self::Certified => "certified",
        }
    }
}

/// Vehicle history keyword filter. Closed vocabulary.
#[derive(Debug, Clone)]
pub enum HistoryKeyword {
    OneOwner,
    CleanTitle,
    Branded,
    Fleet,
}

impl HistoryKeyword {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneOwner => "one_owner",
            Self::CleanTitle => "clean_title",
            Self::Branded => "branded",
            Self::Fleet => "fleet",
        }
    }
}

// ── Domain / input types ──────────────────────────────────────────────────

/// Two-letter dealer state/region code, normalized to uppercase ASCII.
#[derive(Debug, Clone)]
pub struct StateCode(String);

impl StateCode {
    pub fn new(code: &str) -> Result<Self, VisorError> {
        let trimmed = code.trim();
        if trimmed.len() == 2 && trimmed.bytes().all(|b| b.is_ascii_alphabetic()) {
            Ok(Self(trimmed.to_ascii_uppercase()))
        } else {
            Err(VisorError::InvalidFilter {
                message: format!("state code must be two ASCII letters, got: {:?}", trimmed),
            })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Two-letter country code, normalized to uppercase ASCII.
///
/// Only shape is validated (two ASCII letters). Unknown codes such as `ZZ`
/// are accepted; the API is the authority on which values are meaningful.
#[derive(Debug, Clone)]
pub struct CountryCode(String);

impl CountryCode {
    pub fn new(code: &str) -> Result<Self, VisorError> {
        let trimmed = code.trim();
        if trimmed.len() == 2 && trimmed.bytes().all(|b| b.is_ascii_alphabetic()) {
            Ok(Self(trimmed.to_ascii_uppercase()))
        } else {
            Err(VisorError::InvalidFilter {
                message: format!("country code must be two ASCII letters, got: {:?}", trimmed),
            })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// US ZIP code or Canadian postal code.
///
/// **US ZIP**: exactly five ASCII digits; leading zeros are preserved
/// (e.g. `"02134"` stays `"02134"`).
///
/// **Canadian**: `A1A 1A1` (spaced) or compact `A1A1A1`; both are accepted
/// and normalized to the spaced `A1A 1A1` form with uppercase letters.
/// All alphabetic positions reject the letters excluded by Canada Post:
/// `D`, `F`, `I`, `O`, `Q`, `U`.
#[derive(Debug, Clone)]
pub struct PostalCode(String);

/// Returns true if `b` is a valid alphabetic character in a Canadian postal code.
/// Canada Post excludes D, F, I, O, Q, U from all alphabetic positions.
fn is_canadian_postal_alpha(b: u8) -> bool {
    let u = b.to_ascii_uppercase();
    u.is_ascii_alphabetic() && !matches!(u, b'D' | b'F' | b'I' | b'O' | b'Q' | b'U')
}

/// Returns true if `b` is valid as the first letter (FSA) of a Canadian postal code.
/// W and Z are also excluded from the first position in addition to the standard exclusions.
fn is_canadian_fsa_first_alpha(b: u8) -> bool {
    is_canadian_postal_alpha(b) && !matches!(b.to_ascii_uppercase(), b'W' | b'Z')
}

impl PostalCode {
    pub fn new(code: &str) -> Result<Self, VisorError> {
        let code = code.trim();

        // US ZIP: exactly five ASCII digits.
        if code.len() == 5 && code.bytes().all(|b| b.is_ascii_digit()) {
            return Ok(Self(code.to_string()));
        }

        // Canadian: accept compact `A1A1A1` (6 chars) or spaced `A1A 1A1` (7 chars).
        // Both are normalized to the spaced form.
        let compact: Vec<u8> = if code.len() == 6 {
            code.bytes().collect()
        } else if code.len() == 7 && code.as_bytes()[3] == b' ' {
            let mut v: Vec<u8> = code.bytes().collect();
            v.remove(3);
            v
        } else {
            return Err(VisorError::InvalidFilter {
                message: format!("invalid postal code {:?}; expected a 5-digit US ZIP or a Canadian postal code in A1A1A1 or A1A 1A1 form", code),
            });
        };

        // Pattern: alpha digit alpha digit alpha digit
        // Position 0 (FSA first letter) also excludes W and Z.
        let alpha_positions = [0usize, 2, 4];
        let digit_positions = [1usize, 3, 5];
        for &i in &alpha_positions {
            let valid = if i == 0 {
                is_canadian_fsa_first_alpha(compact[i])
            } else {
                is_canadian_postal_alpha(compact[i])
            };
            if !valid {
                let note = if i == 0 {
                    "letters D, F, I, O, Q, U, W, Z are not used in the first position"
                } else {
                    "letters D, F, I, O, Q, U are not used"
                };
                return Err(VisorError::InvalidFilter {
                    message: format!(
                        "invalid character {:?} at position {} of Canadian postal code {:?}; {note}",
                        compact[i] as char, i, code
                    ),
                });
            }
        }
        for &i in &digit_positions {
            if !compact[i].is_ascii_digit() {
                return Err(VisorError::InvalidFilter {
                    message: format!(
                        "expected digit at position {} of Canadian postal code {:?}, got {:?}",
                        i, code, compact[i] as char
                    ),
                });
            }
        }

        let normalized = format!(
            "{}{}{} {}{}{}",
            compact[0].to_ascii_uppercase() as char,
            compact[1] as char,
            compact[2].to_ascii_uppercase() as char,
            compact[3] as char,
            compact[4].to_ascii_uppercase() as char,
            compact[5] as char,
        );
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Geographic latitude in decimal degrees, validated to `-90.0..=90.0`.
#[derive(Debug, Clone)]
pub struct Latitude(f64);

impl Latitude {
    pub fn new(value: f64) -> Result<Self, VisorError> {
        if (-90.0..=90.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(VisorError::InvalidFilter {
                message: format!("latitude must be in -90..=90, got: {}", value),
            })
        }
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

/// Geographic longitude in decimal degrees, validated to `-180.0..=180.0`.
#[derive(Debug, Clone)]
pub struct Longitude(f64);

impl Longitude {
    pub fn new(value: f64) -> Result<Self, VisorError> {
        if (-180.0..=180.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(VisorError::InvalidFilter {
                message: format!("longitude must be in -180..=180, got: {}", value),
            })
        }
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

/// Search radius in miles, validated to `(0.0, 500.0]`.
#[derive(Debug, Clone)]
pub struct RadiusMiles(f64);

impl RadiusMiles {
    pub fn new(miles: f64) -> Result<Self, VisorError> {
        if miles > 0.0 && miles <= 500.0 {
            Ok(Self(miles))
        } else {
            Err(VisorError::InvalidFilter {
                message: format!("radius must be positive and <= 500 miles, got: {}", miles),
            })
        }
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

/// VIN pattern for prefix/wildcard matching.
///
/// VIN characters are `0-9`, `A-Z` excluding `I`, `O`, `Q`.
/// `?` matches exactly one position anywhere in the pattern.
/// `*` may only appear as the final character; a bare `"*"` is intentionally
/// accepted (it matches every VIN) because the contract is structural — `*`
/// at end — not semantic.
#[derive(Debug, Clone)]
pub struct VinPattern(String);

fn is_vin_char(c: char) -> bool {
    // VIN excludes I (between H and J), O (between N and P), and Q (between P and R).
    matches!(c, '0'..='9' | 'A'..='H' | 'J'..='N' | 'P' | 'R'..='Z')
}

impl VinPattern {
    pub fn new(pattern: &str) -> Result<Self, VisorError> {
        let pattern = pattern.trim();
        if pattern.is_empty() {
            return Err(VisorError::InvalidFilter {
                message: "VIN pattern must not be empty".to_string(),
            });
        }

        // '*' may only appear as the final character.
        if let Some(star_pos) = pattern.find('*') {
            if star_pos != pattern.len() - 1 {
                return Err(VisorError::InvalidFilter {
                    message: format!(
                        "'*' may only appear at the end of a VIN pattern, got: {:?}",
                        pattern
                    ),
                });
            }
        }

        // Validate every character in the base (before any trailing '*').
        let base = pattern.strip_suffix('*').unwrap_or(pattern);
        for c in base.chars() {
            if c != '?' && !is_vin_char(c) {
                return Err(VisorError::InvalidFilter {
                    message: format!(
                        "invalid character {:?} in VIN pattern {:?}; \
                         allowed: VIN chars (0-9, A-Z excl. I/O/Q) and '?'",
                        c, pattern
                    ),
                });
            }
        }

        Ok(Self(pattern.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ── Structured relationship enums ──────────────────────────────────────────

/// The anchor point for a radius search — either a postal code or a lat/lon pair.
///
/// Exactly one anchor is required when using `GeoFilter::Radius`.
/// Lat and lon are always supplied together, making a half-specified coordinate
/// impossible to construct.
#[derive(Debug, Clone)]
pub enum GeoOrigin {
    PostalCode(PostalCode),
    Coordinates {
        latitude: Latitude,
        longitude: Longitude,
    },
}

/// Geographic constraint for listing filters.
///
/// Variants are mutually exclusive; the enum enforces this.
///
/// `Origin` anchors distance sorting (`sort=distance`, `distance_miles`)
/// without constraining by radius. `Radius` adds a mile limit. `BBox`
/// constrains to a bounding box.
#[derive(Debug, Clone)]
pub enum GeoFilter {
    Origin(GeoOrigin),
    Radius {
        origin: GeoOrigin,
        miles: RadiusMiles,
    },
    BBox(BBox),
}

/// Inventory mode for listing filters.
///
/// Encodes the three valid states and their associated parameters:
/// - `Active` — default; omits `inventory_status` from the wire request.
/// - `Sold` — emits `inventory_status=sold`; optionally `sold_within_days`.
/// - `Snapshot` — emits `snapshot_date`; active inventory at a point in time.
///
/// Invalid combinations (e.g. `sold_within_days` + active, `snapshot_date` + sold)
/// are impossible to construct.
#[derive(Debug, Clone, Default)]
pub enum InventoryModeFilter {
    #[default]
    Active,
    Sold {
        sold_within_days: Option<u32>,
    },
    Snapshot {
        date: NaiveDate,
    },
}

// ── Unit tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // StateCode

    #[test]
    fn state_code_normalizes_lowercase_to_uppercase() {
        let sc = StateCode::new("ca").unwrap();
        assert_eq!(sc.as_str(), "CA");
    }

    #[test]
    fn state_code_trims_whitespace() {
        let sc = StateCode::new("  TX  ").unwrap();
        assert_eq!(sc.as_str(), "TX");
    }

    #[test]
    fn state_code_accepts_uppercase() {
        assert!(StateCode::new("NY").is_ok());
    }

    #[test]
    fn state_code_rejects_too_long() {
        assert!(StateCode::new("CAL").is_err());
    }

    #[test]
    fn state_code_rejects_too_short() {
        assert!(StateCode::new("C").is_err());
    }

    #[test]
    fn state_code_rejects_digits() {
        assert!(StateCode::new("C1").is_err());
    }

    #[test]
    fn state_code_rejects_empty() {
        assert!(StateCode::new("").is_err());
    }

    #[test]
    fn state_code_rejects_non_ascii_that_uppercases_to_ascii() {
        // "ß".to_uppercase() == "SS" in Unicode, which would pass a naive check.
        assert!(StateCode::new("ß").is_err());
    }

    // CountryCode

    #[test]
    fn country_code_normalizes_lowercase() {
        let cc = CountryCode::new("us").unwrap();
        assert_eq!(cc.as_str(), "US");
    }

    #[test]
    fn country_code_rejects_three_letters() {
        assert!(CountryCode::new("USA").is_err());
    }

    #[test]
    fn country_code_rejects_empty() {
        assert!(CountryCode::new("").is_err());
    }

    #[test]
    fn country_code_rejects_non_ascii_that_uppercases_to_ascii() {
        assert!(CountryCode::new("ß").is_err());
    }

    // PostalCode — US

    #[test]
    fn postal_code_accepts_five_digits() {
        let pc = PostalCode::new("90210").unwrap();
        assert_eq!(pc.as_str(), "90210");
    }

    #[test]
    fn postal_code_preserves_leading_zero() {
        let pc = PostalCode::new("02134").unwrap();
        assert_eq!(pc.as_str(), "02134");
    }

    #[test]
    fn postal_code_trims_whitespace() {
        let pc = PostalCode::new(" 10001 ").unwrap();
        assert_eq!(pc.as_str(), "10001");
    }

    #[test]
    fn postal_code_rejects_four_digits() {
        assert!(PostalCode::new("9021").is_err());
    }

    #[test]
    fn postal_code_rejects_six_all_digits() {
        // Six digits looks like a compact Canadian attempt but position 0 must be alpha.
        assert!(PostalCode::new("902100").is_err());
    }

    #[test]
    fn postal_code_rejects_five_chars_with_letter() {
        assert!(PostalCode::new("9021A").is_err());
    }

    // PostalCode — Canadian

    #[test]
    fn postal_code_accepts_canadian_spaced() {
        let pc = PostalCode::new("K1A 0A9").unwrap();
        assert_eq!(pc.as_str(), "K1A 0A9");
    }

    #[test]
    fn postal_code_accepts_canadian_compact_and_normalizes() {
        let pc = PostalCode::new("K1A0A9").unwrap();
        assert_eq!(pc.as_str(), "K1A 0A9");
    }

    #[test]
    fn postal_code_normalizes_canadian_lowercase() {
        let pc = PostalCode::new("k1a0a9").unwrap();
        assert_eq!(pc.as_str(), "K1A 0A9");
    }

    #[test]
    fn postal_code_normalizes_canadian_spaced_lowercase() {
        let pc = PostalCode::new("k1a 0a9").unwrap();
        assert_eq!(pc.as_str(), "K1A 0A9");
    }

    #[test]
    fn postal_code_rejects_canadian_excluded_letter_d_in_fsa() {
        assert!(PostalCode::new("D1A 0A9").is_err());
    }

    #[test]
    fn postal_code_rejects_canadian_excluded_letter_f() {
        assert!(PostalCode::new("K1F 0A9").is_err());
    }

    #[test]
    fn postal_code_rejects_canadian_excluded_letter_i() {
        assert!(PostalCode::new("K1A 0I9").is_err());
    }

    #[test]
    fn postal_code_rejects_canadian_excluded_letter_o() {
        assert!(PostalCode::new("O1A 0A9").is_err());
    }

    #[test]
    fn postal_code_rejects_canadian_excluded_letter_q() {
        assert!(PostalCode::new("Q1A 0A9").is_err());
    }

    #[test]
    fn postal_code_rejects_canadian_excluded_letter_u() {
        assert!(PostalCode::new("U1A 0A9").is_err());
    }

    #[test]
    fn postal_code_rejects_canadian_w_in_first_position() {
        assert!(PostalCode::new("W1A 0A9").is_err());
    }

    #[test]
    fn postal_code_rejects_canadian_z_in_first_position() {
        assert!(PostalCode::new("Z1A 0A9").is_err());
    }

    #[test]
    fn postal_code_accepts_w_and_z_in_non_first_alpha_positions() {
        // W and Z are only excluded from position 0; positions 2 and 4 allow them.
        assert!(PostalCode::new("K1W 0Z9").is_ok());
    }

    #[test]
    fn postal_code_rejects_wrong_length() {
        assert!(PostalCode::new("K1A0A").is_err()); // 5 chars, not digits
        assert!(PostalCode::new("K1A0A99").is_err()); // 7 chars but no space at index 3
        assert!(PostalCode::new("K1A 0A").is_err()); // 6 chars with space at index 3
    }

    // Latitude

    #[test]
    fn latitude_accepts_zero() {
        assert!(Latitude::new(0.0).is_ok());
    }

    #[test]
    fn latitude_accepts_min_boundary() {
        assert!(Latitude::new(-90.0).is_ok());
    }

    #[test]
    fn latitude_accepts_max_boundary() {
        assert!(Latitude::new(90.0).is_ok());
    }

    #[test]
    fn latitude_rejects_above_90() {
        assert!(Latitude::new(90.001).is_err());
    }

    #[test]
    fn latitude_rejects_below_minus_90() {
        assert!(Latitude::new(-90.001).is_err());
    }

    // Longitude

    #[test]
    fn longitude_accepts_zero() {
        assert!(Longitude::new(0.0).is_ok());
    }

    #[test]
    fn longitude_accepts_min_boundary() {
        assert!(Longitude::new(-180.0).is_ok());
    }

    #[test]
    fn longitude_accepts_max_boundary() {
        assert!(Longitude::new(180.0).is_ok());
    }

    #[test]
    fn longitude_rejects_above_180() {
        assert!(Longitude::new(180.001).is_err());
    }

    #[test]
    fn longitude_rejects_below_minus_180() {
        assert!(Longitude::new(-180.001).is_err());
    }

    // RadiusMiles

    #[test]
    fn radius_accepts_positive_value() {
        assert!(RadiusMiles::new(25.0).is_ok());
    }

    #[test]
    fn radius_accepts_max_boundary() {
        assert!(RadiusMiles::new(500.0).is_ok());
    }

    #[test]
    fn radius_rejects_zero() {
        assert!(RadiusMiles::new(0.0).is_err());
    }

    #[test]
    fn radius_rejects_negative() {
        assert!(RadiusMiles::new(-1.0).is_err());
    }

    #[test]
    fn radius_rejects_above_500() {
        assert!(RadiusMiles::new(500.001).is_err());
    }

    // VinPattern

    #[test]
    fn vin_pattern_accepts_prefix_star() {
        assert!(VinPattern::new("1HGCM82633A*").is_ok());
    }

    #[test]
    fn vin_pattern_accepts_exact_vin() {
        assert!(VinPattern::new("1HGCM82633A004352").is_ok());
    }

    #[test]
    fn vin_pattern_accepts_question_wildcard() {
        assert!(VinPattern::new("1HGCM826??A004352").is_ok());
    }

    #[test]
    fn vin_pattern_accepts_trailing_star_only() {
        assert!(VinPattern::new("1HGCM*").is_ok());
    }

    #[test]
    fn vin_pattern_rejects_star_in_middle() {
        assert!(VinPattern::new("1HG*CM826").is_err());
    }

    #[test]
    fn vin_pattern_bare_star_is_accepted() {
        // A bare "*" satisfies the rule "* may only appear at the end" (index 0
        // == len()-1 when len == 1). It matches every VIN. Intentional — the
        // contract is structural, not semantic. See VinPattern doc comment.
        assert!(VinPattern::new("*").is_ok());
    }

    #[test]
    fn vin_pattern_rejects_illegal_char_i() {
        assert!(VinPattern::new("1HGCM826I3A00435").is_err());
    }

    #[test]
    fn vin_pattern_rejects_illegal_char_o() {
        assert!(VinPattern::new("1HGCM826O3A00435").is_err());
    }

    #[test]
    fn vin_pattern_rejects_illegal_char_q() {
        assert!(VinPattern::new("1HGCM826Q3A00435").is_err());
    }

    #[test]
    fn vin_pattern_rejects_empty() {
        assert!(VinPattern::new("").is_err());
    }

    // Enum wire values

    #[test]
    fn availability_status_wire_values() {
        assert_eq!(AvailabilityStatus::Stock.as_str(), "stock");
        assert_eq!(AvailabilityStatus::Transit.as_str(), "transit");
        assert_eq!(AvailabilityStatus::Build.as_str(), "build");
    }

    #[test]
    fn inventory_type_wire_values() {
        assert_eq!(InventoryType::New.as_str(), "new");
        assert_eq!(InventoryType::Used.as_str(), "used");
        assert_eq!(InventoryType::Certified.as_str(), "certified");
    }

    #[test]
    fn history_keyword_wire_values() {
        assert_eq!(HistoryKeyword::OneOwner.as_str(), "one_owner");
        assert_eq!(HistoryKeyword::CleanTitle.as_str(), "clean_title");
        assert_eq!(HistoryKeyword::Branded.as_str(), "branded");
        assert_eq!(HistoryKeyword::Fleet.as_str(), "fleet");
    }

    #[test]
    fn listing_field_wire_values_spot_check() {
        assert_eq!(ListingField::Make.as_str(), "make");
        assert_eq!(ListingField::Price.as_str(), "price");
        assert_eq!(ListingField::DaysOnMarket.as_str(), "days_on_market");
        assert_eq!(ListingField::Default.as_str(), "default");
        assert_eq!(
            ListingField::DiscountFromMsrp.as_str(),
            "discount_from_msrp"
        );
        assert_eq!(ListingField::OptionsPackages.as_str(), "options_packages");
    }

    // GeoFilter construction

    #[test]
    fn geo_filter_radius_with_postal_origin() {
        let origin = GeoOrigin::PostalCode(PostalCode::new("90210").unwrap());
        let miles = RadiusMiles::new(25.0).unwrap();
        let _filter = GeoFilter::Radius { origin, miles };
    }

    #[test]
    fn geo_filter_radius_with_coordinates_origin() {
        let origin = GeoOrigin::Coordinates {
            latitude: Latitude::new(34.05).unwrap(),
            longitude: Longitude::new(-118.25).unwrap(),
        };
        let miles = RadiusMiles::new(50.0).unwrap();
        let _filter = GeoFilter::Radius { origin, miles };
    }

    #[test]
    fn geo_filter_bbox_construction() {
        let _filter = GeoFilter::BBox(BBox {
            west: -122.5,
            south: 37.2,
            east: -121.9,
            north: 37.8,
        });
    }

    #[test]
    fn geo_filter_origin_with_postal_code() {
        let origin = GeoOrigin::PostalCode(PostalCode::new("90210").unwrap());
        let _filter = GeoFilter::Origin(origin);
    }

    #[test]
    fn geo_filter_origin_with_coordinates() {
        let origin = GeoOrigin::Coordinates {
            latitude: Latitude::new(34.05).unwrap(),
            longitude: Longitude::new(-118.25).unwrap(),
        };
        let _filter = GeoFilter::Origin(origin);
    }

    // InventoryModeFilter construction

    #[test]
    fn inventory_mode_active_is_default() {
        let mode = InventoryModeFilter::default();
        assert!(matches!(mode, InventoryModeFilter::Active));
    }

    #[test]
    fn inventory_mode_sold_with_window() {
        let _mode = InventoryModeFilter::Sold {
            sold_within_days: Some(30),
        };
    }

    #[test]
    fn inventory_mode_sold_without_window() {
        let _mode = InventoryModeFilter::Sold {
            sold_within_days: None,
        };
    }

    #[test]
    fn inventory_mode_snapshot() {
        let _mode = InventoryModeFilter::Snapshot {
            date: NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
        };
    }
}
