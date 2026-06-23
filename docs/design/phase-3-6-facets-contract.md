# Phase 3.6: Facets Contract

This document turns `docs/api/filter-facets.md` into an implementation contract
for the facets-specific pieces of Phase 4. It builds on
`docs/design/phase-3-5-listings-filter-contract.md`; shared listing filters
should keep using `ListingsFilterBase` and the domain types introduced there.

Primary source: `docs/api/filter-facets.md`.

## Design Stance

- Be strict for caller-provided facet fields, metrics, and sorts.
- Reuse `ListingsFilterBase` for all listing-filter parameters accepted by
  `/v1/facets`.
- Keep facet response maps keyed by `String`; response map keys are data from the
  API, not caller-provided input.
- Prefer typed metric parts over raw metric strings because the API documents a
  closed measure vocabulary and a closed aggregate vocabulary.

## Phase 4 Decisions

- Introduce a strict `FacetField` enum for the `facets` request parameter.
- Replace `FacetsFilter.facets: Vec<String>` with `Vec<FacetField>`.
- Replace `FacetsFilter.metric: Option<String>` with
  `Option<FacetMetric>`.
- Keep `FacetSort` as an enum; add `as_str()` and validate metric-sort
  combinations.
- Keep `FacetsFilter::new(facets)` as the constructor and do not derive
  `Default` for `FacetsFilter`, because `facets` is required.
- Keep response structs tolerant:
  `FacetsData.facets`, `FacetsData.range_facets`, `FacetsData.stats`, and
  `FacetsMeta.facets` should remain string-keyed/string-valued where they
  reflect API output.

## Facet Selection

| Wire param | Current shape | Recommended Rust shape | Serialization | Validation |
|---|---|---|---|---|
| `facets` | `Vec<String>` | `Vec<FacetField>` | comma-separated | required; non-empty; enum values only |

`FacetField` wire values:

| Variant | Wire value | Kind |
|---|---|---|
| `Make` | `make` | categorical |
| `Model` | `model` | categorical |
| `InventoryType` | `inventory_type` | categorical |
| `Year` | `year` | categorical |
| `Trim` | `trim` | categorical |
| `Version` | `version` | categorical |
| `BaseExteriorColor` | `base_exterior_color` | categorical |
| `ExteriorColor` | `exterior_color` | categorical |
| `BaseInteriorColor` | `base_interior_color` | categorical |
| `InteriorColor` | `interior_color` | categorical |
| `SeatingCapacity` | `seating_capacity` | categorical |
| `Doors` | `doors` | categorical |
| `Engine` | `engine` | categorical |
| `State` | `state` | categorical |
| `Drivetrain` | `drivetrain` | categorical |
| `AssemblyLocation` | `assembly_location` | categorical |
| `AssemblyCountry` | `assembly_country` | categorical |
| `Transmission` | `transmission` | categorical |
| `FuelType` | `fuel_type` | categorical |
| `BodyType` | `body_type` | categorical |
| `Cylinders` | `cylinders` | categorical |
| `DealerType` | `dealer_type` | categorical |
| `AvailabilityStatus` | `availability_status` | categorical |
| `OptionsPackages` | `options_packages` | categorical |
| `Features` | `features` | categorical |
| `Keywords` | `keywords` | categorical |
| `Price` | `price` | numeric range |
| `Msrp` | `msrp` | numeric range |
| `Miles` | `miles` | numeric range |
| `DaysOnMarket` | `days_on_market` | numeric range |

`FacetField` should expose:

```rust
impl FacetField {
    pub fn as_str(&self) -> &'static str;
    pub fn is_categorical(&self) -> bool;
    pub fn is_numeric_range(&self) -> bool {
        !self.is_categorical()
    }
}
```

Treat `year` as categorical for metric-validation purposes unless API behavior
proves otherwise. The docs call out numeric range facets generally, and the
numeric examples/metrics center on `price`, `msrp`, `miles`, and
`days_on_market`.

## Facet Value Limit

| Wire param | Current shape | Recommended Rust shape | Serialization | Validation |
|---|---|---|---|---|
| `facet_value_limit` | `Option<u32>` | same | decimal string | if set, `<= 100` |

The API default is `20`. Omit this parameter when `None`; do not emit the
default from the SDK unless the caller set it explicitly.

## Metrics

The API accepts `metric=count` by default, or a dot-notation aggregate metric
such as `price.p95` or `days_on_market.median`.

| Wire param | Current shape | Recommended Rust shape | Serialization | Validation |
|---|---|---|---|---|
| `metric` | `Option<String>` | `Option<FacetMetric>` | `count` or `measure.aggregate` | non-count metric requires exactly one categorical facet |

Recommended Rust shape:

```rust
pub enum FacetMetric {
    Count,
    Aggregate {
        measure: FacetMetricMeasure,
        aggregate: FacetMetricAggregate,
    },
}

pub enum FacetMetricMeasure {
    Price,
    Miles,
    Msrp,
    DaysOnMarket,
    DiscountFromMsrp,
}

pub enum FacetMetricAggregate {
    Mean,
    P5,
    P25,
    Median,
    P75,
    P95,
}
```

Metric measure wire values:

| Variant | Wire value |
|---|---|
| `Price` | `price` |
| `Miles` | `miles` |
| `Msrp` | `msrp` |
| `DaysOnMarket` | `days_on_market` |
| `DiscountFromMsrp` | `discount_from_msrp` |

Metric aggregate wire values:

| Variant | Wire value |
|---|---|
| `Mean` | `mean` |
| `P5` | `p5` |
| `P25` | `p25` |
| `Median` | `median` |
| `P75` | `p75` |
| `P95` | `p95` |

Serialization:

- `None` means omit `metric`; the API default is `count`.
- `Some(FacetMetric::Count)` emits `metric=count`.
- `Some(FacetMetric::Aggregate { measure, aggregate })` emits
  `metric={measure}.{aggregate}`.

Validation:

- Non-count metrics require exactly one facet.
- That one facet must be categorical.
- `FacetMetric::Count` can be used with one or more facets.
- `DiscountFromMsrp` is a valid metric measure even though it is not a supported
  facet field.

## Sorting

`FacetSort` already exists. Phase 4 should add `as_str()` and validation.

| Wire param | Current shape | Recommended Rust shape | Serialization | Validation |
|---|---|---|---|---|
| `sort` | `FacetSort` | same | documented wire value | metric sort requires a non-count metric |

`FacetSort` wire values:

| Variant | Wire value |
|---|---|
| `Count` | `count` |
| `CountDesc` | `-count` |
| `Metric` | `metric` |
| `MetricDesc` | `-metric` |

Default sorting is `-count`; `FacetsFilter::new()` should keep initializing
`sort` to `FacetSort::CountDesc`.

Validation:

- `FacetSort::Metric` and `FacetSort::MetricDesc` require
  `Some(FacetMetric::Aggregate { .. })`.
- `FacetSort::Count` and `FacetSort::CountDesc` are valid with no metric,
  `FacetMetric::Count`, or aggregate metrics.

## Shared Listing Filters

`/v1/facets` accepts the same broad filter surface as `/v1/listings`.
`FacetsFilter` should continue to embed `ListingsFilterBase`:

```rust
pub struct FacetsFilter {
    pub base: ListingsFilterBase,
    pub facets: Vec<FacetField>,
    pub facet_value_limit: Option<u32>,
    pub metric: Option<FacetMetric>,
    pub sort: FacetSort,
}
```

Use the Phase 3.5 listings contract for serialization and validation of shared
filter fields:

- vehicle filters
- dealer filters
- inventory mode filters
- numeric range filters
- geo filters
- exclusion filters

Do not duplicate separate facet-specific versions of `PostalCode`,
`GeoFilter`, `InventoryModeFilter`, `HistoryKeyword`, or other shared filter
types.

## Serialization Rules

- Always emit `facets`.
- Always emit `sort`; default is `-count`.
- Emit `facet_value_limit` only when set.
- Emit `metric` only when set.
- Reuse the shared `ListingsFilterBase` serialization logic for all inherited
  listing filters.
- Preserve parameter ordering in tests by returning `Vec<(String, String)>`.

Recommended ordering:

1. `facets`
2. `facet_value_limit`, when set
3. `metric`, when set
4. `sort`
5. shared base filter params in the same order used by `ListingsFilter`

## Validation Rules

Local validation should return `VisorError::InvalidFilter`.

- `facets` must be non-empty.
- `facet_value_limit <= 100` when set.
- Non-count metrics require exactly one facet.
- Non-count metrics require that facet to be categorical.
- `FacetSort::Metric` and `FacetSort::MetricDesc` require a non-count metric.
- Reuse all applicable `ListingsFilterBase` validation rules:
  - `dealer_id.len() <= 50`
  - `vin_pattern.len() <= 10`
  - numeric range pairs have `min <= max`
  - geo constraints are valid
  - inventory-mode constraints are valid
  - empty strings in free-text list filters are rejected

## Response Models

Keep response models tolerant and string-keyed:

- `FacetsData.facets: HashMap<String, Vec<FacetBucket>>`
- `FacetsData.range_facets: HashMap<String, RangeFacet>`
- `FacetsData.stats: HashMap<String, FieldStats>`
- `FacetsMeta.facets: Vec<String>`
- `FacetsMeta.metric: String`
- `FacetsMeta.sort: String`

These fields describe what the API returned. They should not use `FacetField`,
`FacetMetric`, or `FacetSort` unless strict response deserialization becomes an
explicit SDK design goal.

## Phase 4 Test Additions

Add contract tests for:

- all `FacetField::as_str()` wire values
- `FacetField::is_categorical()` and numeric range classification
- `FacetMetric` count serialization
- `FacetMetric` aggregate serialization, e.g. `price.p95`
- `FacetSort` wire values
- `FacetsFilter::new()` default sort `-count`
- empty `facets` validation failure
- `facet_value_limit > 100` validation failure
- non-count metric with zero facets validation failure
- non-count metric with two facets validation failure
- non-count metric with numeric range facet validation failure
- `FacetSort::Metric` without aggregate metric validation failure
