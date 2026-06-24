# Visor Rust SDK — Implementation Handoff

Building a Rust SDK for the Visor Public API (https://api.visor.vin/v1). This document guides implementation of the core SDK architecture.

> **Phase 4 filter note:** The filter examples in this broad handoff predate the
> typed model work completed in Phases 3.5 and 3.6. For current filter shapes,
> validation, serialization, and tests, use
> `docs/design/phase-3-5-listings-filter-contract.md` and
> `docs/design/phase-3-6-facets-contract.md`. Those documents supersede
> conflicting filter examples here.

## Source Of Truth

Use the endpoint docs in `docs/api/` as the primary API contract:

- endpoint paths and methods
- query parameters and wire names
- accepted enum values and validation rules
- response envelope shapes
- documented HTTP status behavior

Use the Python SDK as a secondary compatibility reference for implementation
patterns, especially transport behavior, filter serialization details, and error
mapping. Do not depend on the Python SDK at runtime.

## Tech Stack

- **Async runtime:** tokio
- **HTTP client:** reqwest (with wiremock for mocking in tests)
- **Serialization:** serde + serde_json
- **Error handling:** thiserror
- **Testing:** tokio::test + wiremock
- **Linting/Format:** clippy + rustfmt
- **Type checking:** built-in (strict by default)

## Commands

```bash
# Install dev dependencies
cargo build

# Run tests
cargo test

# Run a single test
cargo test test_name -- --nocapture

# Lint
cargo clippy -- -D warnings

# Format code
cargo fmt

# Type check (implicit with build/test)
cargo check

# Generate docs
cargo doc --open

# Build for release
cargo build --release

# Run examples
cargo run --example basic_listing_search
```

## Architecture Overview

```
src/
  lib.rs                    crate root; re-exports public API
  error.rs                  exception hierarchy (thiserror)
  transport/
    mod.rs                  public re-exports
    async_transport.rs      AsyncVisorTransport (reqwest-based)
    sync_transport.rs       SyncVisorTransport (blocking reqwest)
  client/
    mod.rs                  public re-exports
    async_client.rs         AsyncVisorClient
    sync_client.rs          VisorClient (sync)
  pagination/
    mod.rs                  public re-exports
    async_pagination.rs     paginate_listings, paginate_dealers (futures Stream)
    sync_pagination.rs      iter_listings, iter_dealers (Iterator)
  models/
    mod.rs                  public re-exports
    base.rs                 ListingsFilterBase and shared filter field definitions
    listings.rs             ListingsFilter, ListingSummary, ListingDetail, ListingSnapshot, ListingsPage
    facets.rs               FacetsFilter, FacetBucket, RangeBucket, RangeFacet, FieldStats, FacetsData, FacetsMeta, FacetsResponse
    dealers.rs              DealerFilter, DealerType, DealerSummary, DealerDetail, DealerAddress, DealersPage
    vins.rs                 VinDetail
    usage.rs                UsageSummary, UsageRecord, UsageTotals, UsageMeta
    common.rs               shared types (BBox, VehicleBuild, etc.)

tests/
  common/
    mod.rs                  shared wiremock fixtures (use `mod common;` in each test file)
  test_listings.rs
  test_dealers.rs
  test_vins.rs
  test_facets.rs
  test_usage.rs
```

## Core Abstractions (Ported from Python)

### 1. Dual Async/Sync Pattern

The Python SDK has `AsyncVisorClient` and `VisorClient` (sync). Rust implementation:
- **`AsyncVisorTransport`** — wraps `reqwest::Client`, methods are `async fn` returning `Result<T, VisorError>`
- **`SyncVisorTransport`** — wraps `reqwest::blocking::Client`, methods are regular `fn` returning `Result<T, VisorError>`
- **`AsyncVisorClient`** — owns `AsyncVisorTransport`, public methods are `async fn`
- **`VisorClient`** — owns `SyncVisorTransport`, public methods are regular `fn`

Both clients share the same filter models. Do NOT create separate filter types for sync vs async.

### 2. Filter Models — Structure & Composition

Rust has no inheritance. Choose one of these approaches for filter hierarchy:

**Option A: Explicit Composition (Recommended)**
```rust
pub struct ListingsFilterBase { /* all shared fields */ }

pub struct ListingsFilter {
    pub base: ListingsFilterBase,
    pub limit: u32,           // default 50, max 100; validate() rejects > 100
    pub offset: u32,          // default 0
    pub sort: SortOrder,      // default SortOrder::DaysOnMarket
    pub fields: Option<Vec<String>>,
    pub include: Option<Vec<ListingInclude>>,  // e.g. price_history, options
}

pub struct FacetsFilter {
    pub base: ListingsFilterBase,
    pub facets: Vec<String>,             // required; not Option
    pub facet_value_limit: Option<u32>,
    pub metric: Option<String>,
    pub sort: FacetSort,                 // default FacetSort::CountDesc; always serialized
}
// Do NOT derive Default for FacetsFilter — facets must be non-empty.
// Use FacetsFilter::new(facets) as the primary constructor.
// FacetsFilter.validate() must enforce:
//   - facets is non-empty
//   - all facet names are known (validate against allowed facet list)
//   - facet_value_limit <= 100 if set
//   - metric requires exactly one categorical facet in facets
// These are deliberate ports from Python; do not skip them.
```

Provide a `new` constructor so callers don't have to specify every field. `new` is infallible — validation happens in `validate()`, which client methods call before sending:

```rust
impl FacetsFilter {
    pub fn new(facets: Vec<String>) -> Self {
        Self {
            base: ListingsFilterBase::default(),
            facets,
            facet_value_limit: None,
            metric: None,
            sort: FacetSort::CountDesc,
        }
    }
}
```

Do not make `new` return `Result` — keep construction and validation separate.

`ListingsFilter`, `ListingsFilterBase`, and `DealerFilter` should all derive or implement `Default`. `FacetsFilter` must not — callers must supply `facets` explicitly. The `..Default::default()` pattern in examples applies only to those three types.

```rust
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
```

This is explicit but requires `filter.base.make` for base fields. Do not add `#[serde(flatten)]` to filter structs — serialization is handled manually via `to_params()`, not by deriving `Serialize`.

**Option B: Field Duplication — REJECTED.** All examples in this document use `filter.base.make`; choosing Option B would diverge from every code sample here and require rewriting `to_params()` logic twice. Use Option A.

### 3. Query Serialization & Separator Quirks (CRITICAL)

**All list fields require manual serialization to query string format.** The API does not use JSON encoding; filters must be serialized to `Vec<(String, String)>` for `reqwest`'s query serialization. Use `Vec` (not `HashMap`) — it preserves field ordering in tests and accommodates repeated keys.

Implement a `to_params()` method on each filter type (see Python's [`to_params()` in _base.py](C:\Users\mille\PycharmProjects\visor-api-sdk\src\visor\models\_base.py)):

```rust
impl ListingsFilter {
    pub fn to_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        
        // Standard comma-separated lists (from base)
        if let Some(makes) = &self.base.make {
            params.push(("make".to_string(), makes.join(",")));
        }
        if let Some(features) = &self.base.features {
            params.push(("features".to_string(), features.join(",")));
        }
        
        // Separator quirks (from base)
        if let Some(locs) = &self.base.assembly_location {
            params.push(("assembly_location".to_string(), locs.join("|")));
        }
        if let Some(excludes) = &self.base.exclude_assembly_location {
            params.push(("exclude_assembly_location".to_string(), excludes.join("+")));
        }
        
        // Pagination and sort — always serialized (concrete defaults, not Option)
        params.push(("limit".to_string(), self.limit.to_string()));
        params.push(("offset".to_string(), self.offset.to_string()));
        params.push(("sort".to_string(), self.sort.as_str().to_string()));

        // Optional include — skip if empty (matches Python `if self.include:` behavior)
        if let Some(includes) = &self.include {
            if !includes.is_empty() {
                let s: Vec<&str> = includes.iter().map(|i| i.as_str()).collect();
                params.push(("include".to_string(), s.join(",")));
            }
        }
        
        // Scalar fields from base (PARTIAL EXAMPLE ONLY)
        // to_params() must cover ALL fields from Python's ListingsFilterBase.to_params():
        // bbox, radius, lat/lon, postal_code, inventory_status (omit if Active),
        // snapshot_date, sold_within_days, price ranges, mileage ranges, year ranges,
        // all exclude_* fields, and every other list field in ListingsFilterBase.
        // See C:\Users\mille\PycharmProjects\visor-api-sdk\src\visor\models\_base.py for the full field list.
        if let Some(lat) = self.base.latitude {
            params.push(("latitude".to_string(), lat.to_string()));
        }
        
        params
    }
}
```

Pass the result directly to `reqwest::Client::get(url).query(&filter.to_params())`. This approach gives you full control over formatting and avoids serde's generic JSON encoding.

**`ListingInclude` enum** — define with `as_str()` for serialization:

```rust
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
```

Use the same `as_str()` pattern for `SortOrder` and `FacetSort`. Full `SortOrder` enum:

```rust
pub enum SortOrder {
    DaysOnMarket,     // "days_on_market" (default)
    DaysOnMarketDesc, // "-days_on_market"
    Price,            // "price"
    PriceDesc,        // "-price"
    Miles,            // "miles"
    MilesDesc,        // "-miles"
    Msrp,             // "msrp"
    MsrpDesc,         // "-msrp"
    Discount,         // "discount"
    DiscountDesc,     // "-discount"
    Distance,         // "distance"
}
```

`InventoryStatus` corresponds to Python's `InventoryMode`. Omit from query params when `Active` (API default):

```rust
pub enum InventoryStatus {
    Active,  // "active" — omit from to_params() when this is the value
    Sold,    // "sold"
}
```

`FacetSort` wire values use a leading dash:

```rust
pub enum FacetSort {
    Count,       // "count"
    CountDesc,   // "-count" (default)
    Metric,      // "metric"
    MetricDesc,  // "-metric"
}

impl FacetSort {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Count      => "count",
            Self::CountDesc  => "-count",
            Self::Metric     => "metric",
            Self::MetricDesc => "-metric",
        }
    }
}

impl Default for FacetSort {
    fn default() -> Self { Self::CountDesc }
}
```

### 4. Fields Projection vs Filtering

**`fields` is PROJECTION, not filtering.** It controls which fields are returned in the response.

Filter predicates (`features`, `options_packages`, `make`, `state`, etc.) are different — they narrow the dataset.

The Python SDK validates `fields` against `LISTING_FIELDS` before the request fires. **Deliberate Rust divergence:** skip local field name validation; the API will reject unknown fields with a 400. This avoids maintaining a parallel allowlist in the SDK. Document this decision if asked.

### 5. Fail Before the Wire

Validators should catch these before the HTTP request fires (in a custom validation method, called before sending the request):

- `radius` requires exactly one geo anchor: `postal_code` XOR (`latitude` + `longitude`). Reject if: no anchor, both anchor types present, or only one of lat/lon.
- `bbox` AND `radius` together → return `Err`
- `sold_within_days` without `inventory_status=SOLD` → return `Err`
- `snapshot_date` with `inventory_status != ACTIVE` → return `Err` (snapshot_date only valid when inventory_status=ACTIVE)
- `sold_within_days` AND `snapshot_date` together → return `Err` (mutually exclusive)
- `limit > 100` → return `Err` (enforced on `ListingsFilter`; same rule applies to `DealerFilter`)

Implement these in a `validate()` method on filter types, called explicitly in the client before sending the request. Note: with public struct fields, invalid filters can always be constructed; validation enforces constraints only at request time.

### 6. `DealerFilter`

`DealerFilter` is independent of `ListingsFilterBase`. Key fields and constraints:

```rust
pub enum DealerType {
    Franchise,    // wire value: "franchise"
    Independent,  // wire value: "independent"
}

pub struct DealerFilter {
    pub dealer_id: Option<Vec<String>>,  // max 100 IDs; validate() rejects more
    pub state: Option<Vec<String>>,
    pub country: Option<String>,         // scalar, not Vec; wire param: "country"
    pub dealer_type: Option<DealerType>, // wire param: "type" (not "dealer_type")
    pub make: Option<Vec<String>>,
    pub q: Option<String>,               // free-text search; scalar, no joining
    pub limit: u32,                      // default 50, max 100; validate() rejects > 100
    pub offset: u32,                     // default 0
}
```

`DealerFilter.validate()` must enforce:
- `dealer_id.len() <= 100`
- `limit <= 100`

`to_params()` notes:
- `dealer_type` serializes as `"type"` (not `"dealer_type"`) — use `.push(("type".to_string(), ...))`
- `country` and `q` are scalar strings, pushed directly
- comma-separated lists for `dealer_id`, `state`, `make`
- always emit `limit` and `offset`

**`DealerSummary` response model** — `type` is a reserved keyword in Rust. Use `#[serde(rename = "type")]` on the deserialized field:

```rust
pub struct DealerSummary {
    // ...
    #[serde(rename = "type")]
    pub dealer_type: String,   // "franchise" or "independent"
    // ...
}
```

Do NOT use `r#type` — it compiles but is unergonomic at the call site. Apply the same pattern to `DealerDetail` if it exposes the same field.

### 7. No `**kwargs` Passthrough

The API fails closed on unknown params. Every parameter must be an explicit struct field. Do not implement `serde_json::Value` catchalls or dynamic field addition.

## Error Handling

Use `thiserror` to define the exception hierarchy. All API-originated errors carry a shared body for logging consistency:

```rust
// Normalized internal error shape — NOT the literal JSON response structure.
// The API returns `{"error": {"code": "...", "message": "..."}}` nested under "error".
// The transport extracts body["error"]["code"] and body["error"]["message"],
// and uses the HTTP status code as `status`. ApiErrorBody is assembled in the
// transport's error dispatch, not deserialized directly.
pub struct ApiErrorBody {
    pub status: u16,
    pub code: String,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum VisorError {
    // Missing API key — returned by from_env() when VISOR_API_KEY is not set
    #[error("Missing API key: set VISOR_API_KEY or pass key to new()")]
    MissingApiKey,

    // Pre-wire filter validation failures (local, no HTTP request made)
    #[error("Invalid filter: {message}")]
    InvalidFilter { message: String },

    // HTTP 400
    #[error("Validation error {0.code}: {0.message}")]
    ValidationError(ApiErrorBody),

    // HTTP 401
    #[error("Unauthorized {0.code}: {0.message}")]
    AuthError(ApiErrorBody),

    // HTTP 402
    #[error("Payment required {0.code}: {0.message}")]
    PaymentRequiredError(ApiErrorBody),

    // HTTP 403
    #[error("Forbidden {0.code}: {0.message}")]
    ForbiddenError(ApiErrorBody),

    // HTTP 404
    #[error("Not found {0.code}: {0.message}")]
    NotFoundError(ApiErrorBody),

    // HTTP 429 — retry_after parsed from Retry-After header:
    //   integer seconds → Duration::from_secs(n)
    //   HTTP-date → max(0, date - now) as Duration
    #[error("Rate limited {body.code}: {body.message}; retry after {retry_after:?}")]
    RateLimitError { body: ApiErrorBody, retry_after: Option<Duration> },

    // Any other non-2xx HTTP response
    #[error("API error {0.status} {0.code}: {0.message}")]
    VisorApiError(ApiErrorBody),

    // Network/transport failure (reqwest error — DNS, connection, timeout, etc.)
    #[error("Transport error: {0}")]
    TransportError(#[from] reqwest::Error),

    // Successful HTTP response but body could not be parsed or had unexpected shape.
    // Covers: malformed JSON, missing expected fields, non-object top-level JSON.
    // Corresponds to Python transport raising ValueError/TypeError on response shape.
    #[error("Invalid response: {message}")]
    InvalidResponse { message: String },
}
```

All HTTP variants wrap `ApiErrorBody` for consistent `status`, `code`, and `message` access. Let callers decide retry strategy on `RateLimitError` — do NOT implement auto-retry in transport.

**`TransportError` vs `InvalidResponse` — implementation rule:** `TransportError(#[from] reqwest::Error)` means `?` on a `reqwest` call will automatically convert any `reqwest::Error` into `TransportError`. This includes `response.json::<T>()` failures — deserialization errors from `reqwest` surface as `reqwest::Error`, so using `?` there silently maps them to `TransportError` instead of `InvalidResponse`.

To keep the boundary correct, do not use `?` on `response.json()`. Instead, read the raw bytes and map `serde_json` failures explicitly. The pattern is the same for both transports:

```rust
// Async (AsyncVisorTransport)
let bytes = response.bytes().await?;  // reqwest failure → TransportError via ?
let body: T = serde_json::from_slice(&bytes).map_err(|e| VisorError::InvalidResponse {
    message: format!("Failed to parse response: {e}"),
})?;

// Sync (SyncVisorTransport)
let bytes = response.bytes()?;        // reqwest::blocking failure → TransportError via ?
let body: T = serde_json::from_slice(&bytes).map_err(|e| VisorError::InvalidResponse {
    message: format!("Failed to parse response: {e}"),
})?;
```

`TransportError` = network/connection/timeout failures. `InvalidResponse` = successful HTTP response with unparseable or unexpected body. Apply this pattern in both `async_transport.rs` and `sync_transport.rs`.

**Malformed error bodies on non-2xx responses** — the error dispatch path has the same risk. If the error response body is missing, non-JSON, or not shaped as `{ "error": { "code": "...", "message": "..." } }`, do not propagate a parse error. Instead fall back:

```rust
// Best-effort extraction; never fail on a malformed error body
let (code, message) = match serde_json::from_slice::<serde_json::Value>(&bytes) {
    Ok(v) if v["error"]["code"].is_string() => (
        v["error"]["code"].as_str().unwrap().to_string(),
        v["error"]["message"].as_str().unwrap_or("").to_string(),
    ),
    _ => ("unknown_error".to_string(), String::from_utf8_lossy(&bytes).into_owned()),
};
let body = ApiErrorBody { status: status_code, code, message };
```

Dispatch the resulting `ApiErrorBody` to the appropriate `VisorError` variant by status code as normal. The `"unknown_error"` code and raw body text give callers enough to debug without crashing on unexpected API responses.

## Response Model Deserialization

All response model structs derive `serde::Deserialize`. Collection fields that the API may omit when using field projection must carry `#[serde(default)]` so missing fields deserialize to empty collections rather than failing.

`price_history` differs by model — match the Python defaults exactly:

| Field | Type | Attribute |
|---|---|---|
| `ListingSummary.price_history` | `Vec<PriceHistoryEntry>` | `#[serde(default)]` — defaults to `[]` when absent |
| `ListingDetail.price_history` | `Option<Vec<PriceHistoryEntry>>` | `#[serde(default)]` — defaults to `None` when absent |
| `ListingSnapshot.price_history` | `Option<Vec<PriceHistoryEntry>>` | `#[serde(default)]` — defaults to `None` when absent |

Other fields that need `#[serde(default)]` due to projection or optional API content:

- `ListingSummary.photo_urls: Vec<String>` / `ListingDetail.photo_urls: Vec<String>` — omitted when not projected
- `ListingSnapshot.photo_urls: Vec<String>` — same
- `features: Vec<String>` — omitted when not projected
- `options_packages: Vec<String>` — omitted when not projected
- `options: Vec<VehicleOption>` — omitted unless `include=options`
- `DealerSummary.makes: Vec<String>` — may be absent in API response
- `ListingsPage.meta` / `DealersPage.meta` — pagination meta may be absent on empty result sets; use a loosely-typed map rather than a named struct to avoid chasing the exact shape:
  ```rust
  #[serde(default)]
  pub meta: HashMap<String, serde_json::Value>,
  ```
- `VehicleBuild.window_sticker_verified: bool` — Python defaults to `False`; omitting `#[serde(default)]` on a non-`Option` bool will fail deserialization when the field is absent

`FacetsData` maps must also carry `#[serde(default)]` — Python defaults all three to empty dicts:

```rust
pub struct FacetsData {
    #[serde(default)]
    pub facets: HashMap<String, Vec<FacetBucket>>,
    #[serde(default)]
    pub range_facets: HashMap<String, RangeFacet>,
    #[serde(default)]
    pub stats: HashMap<String, FieldStats>,
}
```

Without `#[serde(default)]`, any projected response that omits these fields will fail deserialization entirely.

Nested types that callers interact with directly (`VehicleBuild`, `VehicleOption`, `VehicleRecord`, `PriceHistoryEntry`, `DealerRef`, `BBox`, `RangeBucket`) must also be re-exported from `lib.rs` — see the Re-exports section below.

## Pagination

Pagination functions return async `Stream` and sync `Iterator` respectively. They internally manage page iteration, calling the client for each page as needed.

### Async Pagination (Futures Stream)

```rust
pub fn paginate_listings(
    client: &AsyncVisorClient,
    filter: ListingsFilter,
    max_pages: Option<usize>,   // None = no limit; Some(0) = no pages fetched
) -> impl futures::stream::Stream<Item = Result<ListingSummary, VisorError>> + '_

pub fn paginate_dealers(
    client: &AsyncVisorClient,
    filter: DealerFilter,
    max_pages: Option<usize>,
) -> impl futures::stream::Stream<Item = Result<DealerSummary, VisorError>> + '_

pub fn paginate_dealer_inventory(
    client: &AsyncVisorClient,
    dealer_id: Uuid,
    filter: ListingsFilter,
    max_pages: Option<usize>,
) -> impl futures::stream::Stream<Item = Result<ListingSummary, VisorError>> + '_
```

Returns a `Stream` of individual items. Built with `futures::stream::unfold` to paginate transparently.

### Sync Pagination (Iterator)

```rust
pub fn iter_listings(
    client: &VisorClient,
    filter: ListingsFilter,
    max_pages: Option<usize>,
) -> impl Iterator<Item = Result<ListingSummary, VisorError>> + '_

pub fn iter_dealers(
    client: &VisorClient,
    filter: DealerFilter,
    max_pages: Option<usize>,
) -> impl Iterator<Item = Result<DealerSummary, VisorError>> + '_

pub fn iter_dealer_inventory(
    client: &VisorClient,
    dealer_id: Uuid,
    filter: ListingsFilter,
    max_pages: Option<usize>,
) -> impl Iterator<Item = Result<ListingSummary, VisorError>> + '_
```

All six helpers take `max_pages: Option<usize>`. `Some(0)` fetches nothing; `None` pages until exhausted. Helpers take the filter by value and own a mutable internal copy that they update (incrementing `offset`) between pages. Do not hold a reference to the caller's filter inside the stream/iterator — take ownership and mutate the internal copy only.

## Public API Surface

### Required Client Methods

Both `AsyncVisorClient` and `VisorClient` must implement all of these (async/sync respectively):

| Method | Signature sketch |
|---|---|
| `filter_listings` | `(filter: &ListingsFilter) -> Result<ListingsPage, VisorError>` |
| `get_listing` | `(id: &str, include: Option<Vec<ListingInclude>>) -> Result<ListingDetail, VisorError>` |
| `lookup_vin` | `(vin: &str, include: Option<Vec<ListingInclude>>) -> Result<VinDetail, VisorError>` |
| `filter_facets` | `(filter: &FacetsFilter) -> Result<FacetsResponse, VisorError>` |
| `search_dealers` | `(filter: &DealerFilter) -> Result<DealersPage, VisorError>` |
| `get_dealer` | `(id: &str) -> Result<DealerDetail, VisorError>` |
| `dealer_inventory` | `(dealer_id: &str, filter: &ListingsFilter) -> Result<ListingsPage, VisorError>` |
| `get_usage` | `(start: Option<NaiveDate>, end: Option<NaiveDate>, metering_class: Option<Vec<String>>) -> Result<UsageSummary, VisorError>` |

`metering_class` is comma-separated in the query string. `filter_listings`, `search_dealers`, and `dealer_inventory` take non-optional filter refs — **deliberate Rust divergence from Python** (which defaults to an empty filter). Callers pass `&ListingsFilter::default()` or `&DealerFilter::default()` explicitly.

Do not ship without all eight methods.

### Response Envelope Decoding

Different endpoints return different envelope shapes — do not assume a uniform wrapper:

| Endpoint type | Response shape | What to return |
|---|---|---|
| `filter_listings`, `search_dealers`, `dealer_inventory` | `{ data: [...], pagination: {...} }` | `ListingsPage` / `DealersPage` (full page struct) |
| `filter_facets` | `{ data: {...}, meta: {...} }` | `FacetsResponse` (full response struct) |
| `get_usage` | `{ data: [...], totals: {...}, meta: {...} }` | `UsageSummary` (full response struct; `data` is `Vec<UsageRecord>`, `totals` is `UsageTotals`) |
| `get_listing`, `get_dealer` | `{ data: {...} }` | unwrap `data` field → `ListingDetail` / `DealerDetail` |
| `lookup_vin` | `{ data: {...} }` | unwrap `data` field → `VinDetail` |

Page/facet/usage methods return the full envelope (callers need pagination metadata). Detail methods unwrap `data` (callers don't need the envelope). Do not double-wrap or return raw `serde_json::Value`.

### Re-exports from `lib.rs`

Only these are public (re-export all from `lib.rs`):
- `VisorClient`, `AsyncVisorClient`, `ClientConfig`
- `VisorError`, `ApiErrorBody`
- All filter types: `ListingsFilter`, `ListingsFilterBase`, `FacetsFilter`, `DealerFilter`
- All enum types used in filter construction: `ListingInclude`, `SortOrder`, `InventoryStatus`, `FacetSort`, `DealerType`
- All response model types: `ListingsPage`, `ListingSummary`, `ListingDetail`, `ListingSnapshot`, `DealersPage`, `DealerSummary`, `DealerDetail`, `DealerAddress`, `FacetsResponse`, `FacetBucket`, `RangeBucket`, `RangeFacet`, `FieldStats`, `FacetsData`, `FacetsMeta`, `VinDetail`, `UsageSummary`, `UsageRecord`, `UsageTotals`, `UsageMeta`
- All nested types that appear in public response model fields: `VehicleBuild`, `VehicleOption`, `VehicleRecord`, `PriceHistoryEntry`, `DealerRef`, `BBox`, `Pagination`, `RangeBucket`
- `paginate_listings`, `iter_listings`, `paginate_dealers`, `iter_dealers`, `paginate_dealer_inventory`, `iter_dealer_inventory`

Internal (keep private via module-level visibility):
- `AsyncVisorTransport`, `SyncVisorTransport` — not re-exported
- `transport` module — private, only `mod.rs` re-exports public types
- `client` module — private, only `mod.rs` re-exports public types

Use `pub(crate)` for internal module items, not `_` prefixes. Idiomatic Rust pattern.

Users import only from the crate root:
```rust
use visor::{VisorClient, ListingsFilter, VisorError};
```

## Standards

### Type Annotations

All public functions must have explicit type annotations. No `.into()` in signatures; use concrete types. Example:

```rust
pub async fn filter_listings(
    &self,
    filter: &ListingsFilter,
) -> Result<ListingsPage, VisorError>
```

### Naming

- Types: `PascalCase`
- Functions/methods: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Module names: `snake_case`

### Documentation

- Top-level doc comments for public types and functions.
- One-liner examples in `//` comments (no multi-paragraph docstrings).
- Example: `/// Filters active listings by make and state.`

### Testing

- Unit tests for serialization logic, filters, error dispatch.
- Integration tests with `wiremock` mocking.
- Tests are in `tests/` directory or co-located with `#[cfg(test)]` modules.
- Use `tokio::test` macro for async tests.

Example with wiremock:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_filter_listings_default() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/listings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({...})))
            .mount(&mock_server)
            .await;

        // Pass the mock server URI as base_url. Transport builds full URLs as:
        //   base_url.trim_end_matches('/') + "/" + path.trim_start_matches('/')
        // Do NOT use Url::join — it drops path segments (/v1 would be lost in prod).
        // In production, base_url defaults to "https://api.visor.vin/v1".
        let client = AsyncVisorClient::with_config(ClientConfig {
            api_key: "test-key".to_string(),
            base_url: mock_server.uri(),
            ..Default::default()
        });
        let filter = ListingsFilter {
            base: ListingsFilterBase::default(),
            limit: 10,
            ..Default::default()
        };
        let result = client.filter_listings(&filter).await;

        assert!(result.is_ok());
    }
}
```

Both clients must support these three constructors:

Use a `ClientConfig` struct to avoid accumulating special-case constructors as options grow (base URL, timeout, future additions like proxy). This keeps the two standard constructors (`new`, `from_env`) ergonomic while giving advanced callers a single escape hatch:

```rust
pub struct ClientConfig {
    pub api_key: String,
    pub base_url: String,                // default: "https://api.visor.vin/v1"
    pub timeout: std::time::Duration,    // default: 30 seconds
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.visor.vin/v1".to_string(),
            timeout: std::time::Duration::from_secs(30),
        }
    }
}

impl AsyncVisorClient {
    // Explicit key; panics if api_key is empty (document this)
    pub fn new(api_key: String) -> Self

    // Reads VISOR_API_KEY env var; returns Err(VisorError::MissingApiKey) if absent
    pub fn from_env() -> Result<Self, VisorError>

    // Full control over base URL + timeout; used in tests (base_url) and production (timeout).
    // Panics if config.api_key is empty — same contract as new().
    pub fn with_config(config: ClientConfig) -> Self
}
```

`new` and `from_env` are convenience wrappers that construct a `ClientConfig` with defaults and call `with_config`. `with_config` panics if `config.api_key` is empty — the same contract as `new`. This makes `ClientConfig::default()` safe to use as a starting point (the empty `api_key` placeholder is expected to be filled in) while still catching the mistake at construction time. `from_env` returns `Err(VisorError::MissingApiKey)` rather than panicking because the absence of an env var is a recoverable configuration error at startup. Tests set `base_url` to the wiremock server URI. Production callers who need a non-default timeout set `timeout`. Both can be overridden simultaneously via `with_config`.

`VisorClient` (sync) exposes the same three constructors. All take `String` (not `impl Into<String>`) — consistent with the "concrete types in signatures" standard. Callers use `.to_string()` or `String::from(...)` at the call site.

The Python SDK defaults to `timeout=30.0` seconds. The Rust transport passes `config.timeout` to `reqwest::ClientBuilder::timeout()` at construction time. `ClientConfig` must also be re-exported from `lib.rs` so callers can construct it without reaching into internal modules.

### Dependencies

Keep the dependency tree lean. Currently:
- `tokio` (async runtime)
- `reqwest` (HTTP)
- `serde` + `serde_json` (serialization)
- `thiserror` (errors)
- `wiremock` (test mocking)
- `chrono` (date/time types)

No additional dependencies without justification.

## Development Workflow

1. **Branch:** Create a feature branch (e.g., `feat/listings-filter`)
2. **Code:** Implement one module at a time
3. **Test:** Write unit + integration tests as you go
4. **Lint:** Run `cargo clippy -- -D warnings`
5. **Format:** Run `cargo fmt`
6. **Commit:** Small, atomic commits with clear messages
7. **PR:** Open a PR with a clear summary

## Integration Test Release Gates

Tests marked with `#[ignore]` are live API tests that require `VISOR_API_KEY` env var (separate from mocked unit tests):
- `test_int_listings` — live API call to filter listings
- `test_int_dealers` — live API call to search dealers
- `test_int_vins` — live API call to lookup VIN
- `test_int_facets` — live API call to get facets
- `test_int_usage` — live API call to get usage

Bash: `VISOR_API_KEY=... cargo test -- --ignored`
PowerShell: `$env:VISOR_API_KEY="..."; cargo test -- --ignored`

These should NOT run in CI unless explicitly configured. Unit/integration tests with wiremock mocks run normally with `cargo test`.

## Known Constraints

### No Builder Pattern for Filters

Filters are intentionally verbose. Users construct them explicitly using struct literals and `Default::default()` for unset fields. Small required-field constructors like `FacetsFilter::new(facets)` are allowed; do not add chained builder APIs (`.with_make(...)`, `.with_state(...)`, etc.).

```rust
let filter = ListingsFilter {
    base: ListingsFilterBase {
        make: Some(vec!["Toyota".to_string()]),
        state: Some(vec!["CA".to_string()]),
        ..Default::default()
    },
    ..Default::default()
};
```

This is intentional — it makes filter intent visible without a builder API.

### Inventory Status Omission

`inventory_status=active` is the API default and should be omitted from URLs when `inventory_status` is `InventoryStatus::Active`. This keeps URLs clean.

### Snapshot Date Validation

`snapshot_date` is only valid when `inventory_status=ACTIVE`. The validation must happen before the request fires, in `ListingsFilter::validate()`.

## Cargo.toml Skeleton

```toml
[package]
name = "visor"
version = "0.1.0"
edition = "2021"
# rust-version = "1.XX"  # set after confirming MSRV against reqwest 0.13 and thiserror 2.0

# Versions listed are minimum starting points; prefer current stable compatible versions.
[dependencies]
tokio = { version = "1.52", features = ["full"] }
reqwest = { version = "0.13", features = ["json", "blocking"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
chrono = { version = "0.4", features = ["serde"] }
futures = "0.3"

[dev-dependencies]
wiremock = "0.6"
tokio = { version = "1.52", features = ["full"] }
serde_json = "1.0"
```

## Next Steps

1. Set up the basic cargo project structure
2. Implement error types (`error.rs`)
3. Implement filter models (`models/`) with serialization logic
4. Implement transport layer (`transport/`)
5. Implement client layer (`client/`)
6. Implement pagination (`pagination/`)
7. Add integration tests with wiremock mocking
8. Document examples

Each step should be fully tested before moving to the next.
