# Phase 3.5: Listings Filter Contract

This document turns `docs/api/filter-listings.md` into an implementation
contract for Phase 4. The goal is to decide which filter fields should become
typed Rust inputs, which rules should be validated locally, and which
serialization quirks must be preserved.

Primary source: `docs/api/filter-listings.md`.

## Design Stance

- Be strict for caller-provided filter inputs.
- Be tolerant for API response models unless the API contract requires strict
  response deserialization.
- Prefer Rust types/enums that make invalid filter combinations impossible when
  ergonomics stay reasonable.
- Use explicit `validate()` checks for relationship rules that do not fit cleanly
  into the type model.
- Normalize user-provided code fields where normalization is obvious and safe
  (`StateCode::new("ca")` stores `CA`).

## Phase 4 Decisions

- Introduce a strict `ListingField` enum for projection fields.
- Introduce a `VinPattern` domain type instead of validating raw strings only.
- Add the `uuid` crate and use `uuid::Uuid` for `dealer_id` filter inputs.
- Implement `GeoFilter` and `InventoryModeFilter` now rather than carrying
  invalid combinations as separate fields.
- Introduce typed code/domain fields where the API documents shape rules:
  `StateCode`, `CountryCode`, `PostalCode`, `Latitude`, `Longitude`, and
  `RadiusMiles`.
- Normalize `StateCode` and `CountryCode` to uppercase after trimming. Validate
  them as exactly two ASCII letters.
- Normalize `PostalCode` by trimming only. Validate it as exactly five ASCII
  digits and store it as a string so leading zeros are preserved.
- Keep response models tolerant; these stricter types are for caller-provided
  filters.

## Pagination And Sorting

| Wire param | Current shape | Recommended Rust shape | Serialization | Validation |
|---|---|---|---|---|
| `limit` | `u32` | `u32` | decimal string | default `50`, max `100` |
| `offset` | `u32` | `u32` | decimal string | zero-based; `u32` prevents negative values |
| `sort` | `SortOrder` | `SortOrder` | documented wire value | `distance` requires a geo origin |

Sort wire values:

| Variant | Wire value |
|---|---|
| `DaysOnMarket` | `days_on_market` |
| `DaysOnMarketDesc` | `-days_on_market` |
| `Price` | `price` |
| `PriceDesc` | `-price` |
| `Miles` | `miles` |
| `MilesDesc` | `-miles` |
| `Msrp` | `msrp` |
| `MsrpDesc` | `-msrp` |
| `Discount` | `discount` |
| `DiscountDesc` | `-discount` |
| `Distance` | `distance` |

Default sorting is `days_on_market`.

## Response Projection

| Wire param | Current shape | Recommended Rust shape | Serialization | Validation |
|---|---|---|---|---|
| `fields` | `Option<Vec<String>>` | `Option<Vec<ListingField>>` | comma-separated | enum values only |
| `include` | `Option<Vec<ListingInclude>>` | `Option<Vec<ListingInclude>>` | comma-separated | enum values only |

`include` wire values:

| Variant | Wire value |
|---|---|
| `PriceHistory` | `price_history` |
| `Options` | `options` |

`ListingField` should be strict. Do not add a `Custom(String)` escape hatch in
Phase 4; update the SDK enum when the API adds supported projection fields.

## Vehicle Filters

Most vehicle filters are free-text or API-controlled vocabulary strings and
should serialize as comma-separated lists.

| Wire param | Current shape | Recommended Rust shape | Serialization | Validation |
|---|---|---|---|---|
| `make` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `model` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `trim` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `year` | `Option<Vec<i32>>` | `Option<Vec<u16>>` or `Option<Vec<u32>>` | comma-separated | sensible model-year range |
| `body_type` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `transmission` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `drivetrain` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `fuel_type` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `powertrain_type` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `engine` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `version` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exterior_color` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `interior_color` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `base_exterior_color` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `base_interior_color` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `seating_capacity` | `Option<Vec<i32>>` | `Option<Vec<u8>>` | comma-separated | positive values |
| `cylinders` | `Option<Vec<i32>>` | `Option<Vec<u8>>` | comma-separated | positive values |
| `doors` | `Option<Vec<i32>>` | `Option<Vec<u8>>` | comma-separated | positive values |
| `options_packages` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `features` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `assembly_location` | `Option<Vec<String>>` | same | pipe-separated | non-empty values |
| `assembly_country` | `Option<Vec<String>>` | `Option<Vec<CountryCode>>` | comma-separated | two ASCII letters; normalize to uppercase |
| `vin_pattern` | `Option<Vec<String>>` | `Option<Vec<VinPattern>>` | comma-separated | up to 10 patterns; `?` matches one position; `*` allowed only at end |
| `keywords` | `Option<Vec<String>>` | `Option<Vec<HistoryKeyword>>` | comma-separated | enum values only |

`keywords` wire values:

| Variant | Wire value |
|---|---|
| `OneOwner` | `one_owner` |
| `CleanTitle` | `clean_title` |
| `Branded` | `branded` |
| `Fleet` | `fleet` |

## Dealer Filters

| Wire param | Current shape | Recommended Rust shape | Serialization | Validation |
|---|---|---|---|---|
| `state` | `Option<Vec<String>>` | `Option<Vec<StateCode>>` | comma-separated | two ASCII letters; normalize to uppercase |
| `dealer_id` | `Option<Vec<String>>` | `Option<Vec<uuid::Uuid>>` | comma-separated | up to 50 UUIDs |
| `dealer_type` | `Option<Vec<String>>` | `Option<Vec<DealerType>>` | comma-separated | enum values only |

`dealer_type` wire values:

| Variant | Wire value |
|---|---|
| `Franchise` | `franchise` |
| `Independent` | `independent` |

Add `uuid` as a dependency in Phase 4. The API docs identify dealer IDs as UUIDs,
and using `uuid::Uuid` is clearer than hand-validating strings.

## Inventory Status Filters

The API exposes three separate wire params, but the valid combinations are
structured:

| Wire param | Current shape | Recommended Rust shape | Serialization | Validation |
|---|---|---|---|---|
| `availability_status` | `Option<Vec<String>>` | `Option<Vec<AvailabilityStatus>>` | comma-separated | enum values only |
| `inventory_type` | `Option<Vec<String>>` | `Option<Vec<InventoryType>>` | comma-separated | enum values only |
| `inventory_status` | `InventoryStatus` | inside `InventoryModeFilter` | omit active by default; emit `sold` when needed | represented by enum shape |
| `sold_within_days` | `Option<i32>` | inside `InventoryModeFilter::Sold` | decimal string | positive; sold mode only |
| `snapshot_date` | `Option<NaiveDate>` | inside `InventoryModeFilter::Snapshot` | `YYYY-MM-DD` | active snapshot mode only |

Closed values:

| Field | Variant | Wire value |
|---|---|---|
| `availability_status` | `Stock` | `stock` |
| `availability_status` | `Transit` | `transit` |
| `availability_status` | `Build` | `build` |
| `inventory_type` | `New` | `new` |
| `inventory_type` | `Used` | `used` |
| `inventory_type` | `Certified` | `certified` |
| `inventory_status` | `Active` | `active` |
| `inventory_status` | `Sold` | `sold` |

Use `InventoryType::Certified` for the typed API and serialize it as
`certified`. Do not add a `Cpo` alias variant in Phase 4 unless compatibility
tests require it.

Phase 4 Rust shape:

```rust
pub enum InventoryModeFilter {
    Active,
    Sold {
        sold_within_days: Option<u32>,
    },
    Snapshot {
        date: NaiveDate,
    },
}
```

This makes invalid combinations unrepresentable:

- `sold_within_days` with active inventory
- `snapshot_date` with sold inventory
- `sold_within_days` and `snapshot_date` together

Do this in Phase 4. This enum replaces the separate public filter fields for
`inventory_status`, `sold_within_days`, and `snapshot_date`.

## Numeric Range Filters

| Wire param | Current shape | Recommended Rust shape | Serialization | Validation |
|---|---|---|---|---|
| `min_price` | `Option<i32>` | `Option<u32>` | decimal string | `min_price <= max_price` |
| `max_price` | `Option<i32>` | `Option<u32>` | decimal string | `min_price <= max_price` |
| `min_mileage` | `Option<i32>` | `Option<u32>` | decimal string | `min_mileage <= max_mileage` |
| `max_mileage` | `Option<i32>` | `Option<u32>` | decimal string | `min_mileage <= max_mileage` |
| `min_msrp` | `Option<i32>` | `Option<u32>` | decimal string | `min_msrp <= max_msrp` |
| `max_msrp` | `Option<i32>` | `Option<u32>` | decimal string | `min_msrp <= max_msrp` |
| `min_days_on_market` | `Option<i32>` | `Option<u32>` | decimal string | `min_days_on_market <= max_days_on_market` |
| `max_days_on_market` | `Option<i32>` | `Option<u32>` | decimal string | `min_days_on_market <= max_days_on_market` |

Use unsigned types for caller-provided nonnegative bounds. Relationship checks
still belong in `validate()`.

## Geographic Filters

The API exposes separate wire params, but the valid combinations are structured:

| Wire param | Current shape | Recommended Rust shape | Serialization | Validation |
|---|---|---|---|---|
| `latitude` | `Option<f64>` | `Latitude` inside `GeoOrigin` | decimal string | `-90..=90` |
| `longitude` | `Option<f64>` | `Longitude` inside `GeoOrigin` | decimal string | `-180..=180` |
| `postal_code` | `Option<String>` | `PostalCode` inside `GeoOrigin` | string | exactly five ASCII digits |
| `radius` | `Option<f64>` | `RadiusMiles` inside `GeoFilter::Radius` | decimal string | positive; max `500`; requires origin |
| `bbox` | `Option<BBox>` | `GeoFilter::BBox(BBox)` | `west,south,east,north` | cannot combine with radius; max diagonal `1000` miles |

Phase 4 Rust shape:

```rust
pub struct Latitude(f64);
pub struct Longitude(f64);
pub struct PostalCode(String);
pub struct RadiusMiles(f64);

pub enum GeoOrigin {
    PostalCode(PostalCode),
    Coordinates {
        latitude: Latitude,
        longitude: Longitude,
    },
}

pub enum GeoFilter {
    Radius {
        origin: GeoOrigin,
        miles: RadiusMiles,
    },
    BBox(BBox),
}
```

This encodes:

- `radius` requires exactly one origin: postal code or latitude/longitude
- latitude and longitude are supplied together
- `bbox` and `radius` are mutually exclusive

`PostalCode` stores a string, not a number, so leading zeros are preserved.

Phase 4 should validate the documented `bbox` max diagonal of 1000 miles using a
lightweight local calculation, not a GIS dependency. A reasonable approximation:

```rust
fn bbox_diagonal_miles(west: f64, south: f64, east: f64, north: f64) -> f64 {
    let miles_per_degree_lat = 69.0;
    let mid_lat_rad = ((north + south) / 2.0).to_radians();

    let height = (north - south).abs() * miles_per_degree_lat;
    let width = (east - west).abs() * miles_per_degree_lat * mid_lat_rad.cos();

    (width.powi(2) + height.powi(2)).sqrt()
}
```

This is approximate, but it is sufficient for enforcing the API's coarse
1000-mile guardrail. Avoid turning the core SDK into a general geospatial
mapping library. Address geocoding, reverse geocoding, point-in-state lookup,
ocean/land detection, market-coverage datasets, and address-to-bbox helpers
belong in a possible future optional package or crate, not in core filter
validation.

## Exclusion Filters

Most exclusion filters mirror positive filters and serialize as comma-separated
lists.

| Wire param | Current shape | Recommended Rust shape | Serialization | Validation |
|---|---|---|---|---|
| `exclude_make` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exclude_model` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exclude_trim` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exclude_year` | `Option<Vec<i32>>` | `Option<Vec<u16>>` or `Option<Vec<u32>>` | comma-separated | sensible model-year range |
| `exclude_state` | `Option<Vec<String>>` | `Option<Vec<StateCode>>` | comma-separated | two ASCII letters; normalize to uppercase |
| `exclude_inventory_type` | `Option<Vec<String>>` | `Option<Vec<InventoryType>>` | comma-separated | enum values only |
| `exclude_body_type` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exclude_transmission` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exclude_drivetrain` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exclude_version` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exclude_engine` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exclude_assembly_location` | `Option<Vec<String>>` | same | plus-separated | non-empty values |
| `exclude_assembly_country` | `Option<Vec<String>>` | `Option<Vec<CountryCode>>` | comma-separated | two ASCII letters; normalize to uppercase |
| `exclude_exterior_color` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exclude_interior_color` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exclude_base_exterior_color` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exclude_base_interior_color` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exclude_options_packages` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exclude_features` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exclude_fuel_type` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exclude_powertrain_type` | `Option<Vec<String>>` | same | comma-separated | non-empty values |
| `exclude_keywords` | `Option<Vec<String>>` | `Option<Vec<HistoryKeyword>>` | comma-separated | enum values only |

## Serialization Rules

- Emit `limit`, `offset`, and `sort` for default listing filters.
- Omit `inventory_status=active` by default.
- Emit `inventory_status=sold` when sold mode is selected.
- Serialize most list fields with commas.
- Serialize `assembly_location` with pipes.
- Serialize `exclude_assembly_location` with plus signs.
- Serialize `bbox` as `west,south,east,north`.
- Serialize dates as ISO 8601 `YYYY-MM-DD`.
- Preserve parameter ordering in tests by returning `Vec<(String, String)>`.

## Validation Rules

Local validation should return `VisorError::InvalidFilter`.

- `limit <= 100`
- `dealer_id.len() <= 50`
- `vin_pattern.len() <= 10`
- VIN patterns may contain VIN characters, `?`, and a terminal `*`; `*` cannot
  appear in the middle.
- `state` and `exclude_state` are two-letter codes.
- `postal_code` is exactly five ASCII digits.
- `latitude` is in `-90..=90`.
- `longitude` is in `-180..=180`.
- `radius` is positive and `<= 500`.
- `radius` requires exactly one origin: postal code or latitude/longitude.
- `bbox` and `radius` are mutually exclusive.
- `bbox` diagonal is at most 1000 miles.
- `sort=distance` requires a geo origin.
- `sold_within_days` requires sold inventory mode.
- `snapshot_date` requires active/snapshot inventory mode.
- `sold_within_days` and `snapshot_date` are mutually exclusive.
- For each numeric range pair, `min <= max`.
- Empty strings in list filters should be rejected.
