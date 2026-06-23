# Cleanup Notes

Small follow-up notes that are worth revisiting but do not need to block the
current phase.

## DealerFilter query field

- `DealerFilter` currently exposes `q: Option<String>`.
- `q` is a common API/query-string convention for free-text search, but it is
  terse for a public Rust SDK field.
- Consider renaming the Rust field to `query: Option<String>` while still
  serializing it as the wire parameter `q` in `DealerFilter::to_params()`.

## Domain types for validated input

- Consider introducing small domain types for user-provided filter values that
  have universal validity rules, such as latitude and longitude.
- Possible examples: `Latitude`, `Longitude`, `CountryCode`, `RegionCode`.
- Prefer using these to validate caller input before sending a request, where
  they make the SDK easier and safer to use.
- Be more tolerant when deserializing API responses. The SDK should not become
  an accidental data-quality gate for values returned by the API unless the API
  contract explicitly requires that behavior.
- Latitude/longitude are likely the strongest candidates because their valid
  ranges are universal. State/country codes may need looser shape validation or
  more API-specific investigation before becoming strict types.

## Listing filter validation and numeric types

- `ListingsFilterBase` has several user-provided fields that should likely get
  input validation before requests are sent: `year`, `state`,
  `availability_status`, `inventory_type`, `vin_pattern`, `postal_code`,
  `latitude`, and `longitude`.
- Consider using unsigned numeric types for filter bounds that cannot be
  negative, such as min/max price, MSRP, mileage, and days on market.
- Even with unsigned types, keep explicit validation for relationships such as
  `min_price <= max_price`, `min_mileage <= max_mileage`, and valid geospatial
  combinations.
- Keep response model choices separate from input model choices. API responses
  may remain more permissive, while caller-provided filters can be stricter.

## Filter enums for closed vocabularies

- During the Phase 4 model/filter pass, consider replacing stringly typed input
  fields with enums where `docs/api/` defines a closed vocabulary.
- Strong candidates include `availability_status` (`stock`, `transit`,
  `build`), `inventory_type` (`new`, `used`, `certified`), `dealer_type`
  (`franchise`, `independent`), `keywords` (`one_owner`, `clean_title`,
  `branded`, `fleet`), and any other documented closed values.
- Prefer these stricter enums for caller-provided filter inputs so typos are
  caught before making a request.
- Keep response models more tolerant unless strict response deserialization is
  an explicit SDK design decision.

## Geo filter type modeling

- `bbox` and `radius` cannot be combined according to the API docs.
- The Python SDK currently enforces this with a model validator:
  `radius` requires exactly one of `postal_code` or `latitude + longitude`, and
  `bbox` is mutually exclusive with `radius`.
- Consider replacing separate `postal_code`, `latitude`, `longitude`, `radius`,
  and `bbox` filter fields with a more structured input type for geospatial
  filtering.
- A possible shape:

```rust
pub enum GeoOrigin {
    PostalCode(String),
    Coordinates {
        latitude: Latitude,
        longitude: Longitude,
    },
}

pub enum GeoFilter {
    Radius {
        origin: GeoOrigin,
        miles: f64,
    },
    BBox(BBox),
}
```

- Then `ListingsFilterBase` could expose `geo: Option<GeoFilter>`, making
  invalid `radius + bbox` combinations unrepresentable.
- This would also encode the rule that radius searches require either a postal
  code or latitude/longitude origin.
- Revisit in Phase 4 alongside filter validation, domain types, and query
  serialization ergonomics.

## Inventory mode type modeling

- The Python SDK validator also enforces relationships between
  `inventory_status`, `sold_within_days`, and `snapshot_date`.
- Current logical rules:
  - `sold_within_days` requires `inventory_status = sold`
  - `snapshot_date` requires `inventory_status = active`
  - `sold_within_days` and `snapshot_date` are mutually exclusive
- Consider replacing the separate fields with an enum that represents valid
  inventory modes directly:

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

- This would make invalid combinations such as `sold_within_days` with active
  inventory, `snapshot_date` with sold inventory, or both historical modes at
  once unrepresentable in normal construction.
- Revisit the exact shape against `docs/api/`: the API wire value still uses
  `inventory_status=active|sold`, plus optional `sold_within_days` or
  `snapshot_date` query params.
- If this enum feels too large for Phase 4, keep explicit `validate()` checks
  that mirror the Python SDK validator, but prefer the enum if ergonomics stay
  reasonable.

## Fallible constructors and panic boundaries

- Current constructors use panics for some setup-time failures:
  `with_config()` asserts that `api_key` is non-empty, and transport
  construction calls `expect()` if `reqwest` client construction fails.
- This is acceptable for early scaffolding, but production-grade SDKs should be
  careful about panics because an SDK panic can crash the caller's application.
- Consider adding fallible constructors such as:

```rust
pub fn try_new(api_key: String) -> Result<Self, VisorError>
pub fn try_with_config(config: ClientConfig) -> Result<Self, VisorError>
```

- These should return a typed error for invalid configuration or client
  construction failures instead of panicking.
- Existing panicking constructors could remain as convenience wrappers, or the
  public API could move fully toward fallible construction.
- Keep the boundary clear: request-time failures already return
  `Result<_, VisorError>` and should continue to do so. Panics should be
  reserved for internal invariant violations or explicitly documented
  convenience methods.
