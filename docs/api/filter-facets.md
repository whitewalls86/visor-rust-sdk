# Filter Facets

**Endpoint:** `GET /v1/facets`

Returns categorical facets, numeric range facets, and stats for an explicit listing filter surface facet selection. Use `facets=make,model,price`.

**Supported facets:** `make`, `model`, `inventory_type`, `year`, `trim`, `version`, `base_exterior_color`, `exterior_color`, `base_interior_color`, `interior_color`, `seating_capacity`, `doors`, `engine`, `state`, `drivetrain`, `assembly_location`, `assembly_country`, `transmission`, `fuel_type`, `body_type`, `cylinders`, `dealer_type`, `availability_status`, `options_packages`, `features`, `keywords`, `price`, `msrp`, `miles`, `days_on_market`

---

## Authorization

| Header | Type | Required | Description |
|--------|------|----------|-------------|
| `Authorization` | string | Yes | Send API keys as `Authorization: Bearer <api_key>`. Query-string API keys are rejected. |

---

## Example Request

```bash
curl --request GET \
  --url https://api.visor.vin/v1/facets \
  --header 'Authorization: Bearer <token>'
```

---

## Example Response (200)

```json
{
  "data": {
    "total": 62152,
    "facets": {
      "make": [
        {
          "value": "Toyota",
          "count": 62152
        }
      ],
      "model": [
        {
          "value": "Camry",
          "count": 13730
        },
        {
          "value": "Tacoma",
          "count": 10682
        },
        {
          "value": "RAV4",
          "count": 7499
        }
      ]
    },
    "range_facets": {
      "price": {
        "buckets": [
          {
            "min": 24608,
            "max": 28732,
            "count": 5270
          },
          {
            "min": 28732,
            "max": 32856,
            "count": 5528
          },
          {
            "min": 32856,
            "max": 36980,
            "count": 8392
          }
        ],
        "interval": 4124,
        "min": 3988,
        "max": 86468
      }
    },
    "stats": {
      "price": {
        "min": 0,
        "max": 148085,
        "count": 60556,
        "missing": 1596,
        "mean": 38889.62,
        "median": 38300,
        "stddev": 15641.34
      }
    }
  },
  "meta": {
    "facets": [
      "make",
      "model",
      "price"
    ],
    "metric": "count",
    "sort": "-count",
    "minimum_metric_count": 10
  }
}
```

---

## Query Parameters

### Facet Selection & Behavior

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `facets` | string | **Yes** | Required comma-separated facet names to return. Supported facets: `make`, `model`, `inventory_type`, `year`, `trim`, `version`, `base_exterior_color`, `exterior_color`, `base_interior_color`, `interior_color`, `seating_capacity`, `doors`, `engine`, `state`, `drivetrain`, `assembly_location`, `assembly_country`, `transmission`, `fuel_type`, `body_type`, `cylinders`, `dealer_type`, `availability_status`, `options_packages`, `features`, `keywords`, `price`, `msrp`, `miles`, `days_on_market`. |
| `facet_value_limit` | string | No | Maximum number of values returned per categorical facet. Defaults to `20`; maximum `100`. Numeric range facets always return fixed buckets. |
| `metric` | string | No | Facet metric used to compute a per-bucket aggregate. Defaults to `count`. Supported measures: `price`, `miles`, `msrp`, `days_on_market`, `discount_from_msrp`. Supported aggregates: `mean`, `p5`, `p25`, `median`, `p75`, `p95`. Use dot notation, e.g., `price.p95` or `days_on_market.median`. Non-count metrics require exactly one categorical facet. |
| `sort` | enum | No | Facet bucket ordering. Defaults to `-count`. Sorting by `metric` requires a non-count metric. Options: `count`, `-count`, `metric`, `-metric`. |

### Vehicle Filters (applied before counting buckets)

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `make` | string | No | Comma-separated make names or slugs. |
| `model` | string | No | Comma-separated model names or slugs. |
| `trim` | string | No | Comma-separated trim names. |
| `year` | string | No | Comma-separated model years. |
| `body_type` | string | No | Comma-separated body types. |
| `transmission` | string | No | Comma-separated transmission values. |
| `drivetrain` | string | No | Comma-separated drivetrain values. |
| `fuel_type` | string | No | Comma-separated fuel type values. |
| `powertrain_type` | string | No | Comma-separated powertrain type values. |
| `engine` | string | No | Comma-separated engine descriptions. |
| `version` | string | No | Comma-separated vehicle version values. |
| `exterior_color` | string | No | Comma-separated exterior color values. |
| `interior_color` | string | No | Comma-separated interior color values. |
| `base_exterior_color` | string | No | Comma-separated normalized exterior color values. |
| `base_interior_color` | string | No | Comma-separated normalized interior color values. |
| `seating_capacity` | string | No | Comma-separated seating capacity integers. |
| `cylinders` | string | No | Comma-separated cylinder count integers. |
| `doors` | string | No | Comma-separated door count integers. |
| `options_packages` | string | No | Comma-separated manufacturer option/package codes. |
| `features` | string | No | Comma-separated feature tokens. |
| `assembly_location` | string | No | Pipe-separated assembly locations. Uses `\|` because locations often contain commas. |
| `assembly_country` | string | No | Comma-separated assembly country values. |
| `vin_pattern` | string | No | Comma-separated VIN masks, up to 10 distinct patterns. VIN characters match themselves, `?` matches one VIN position, and `*` is allowed only at the end. Short masks are treated as prefixes, e.g., `1HG` or `1HGCM826*`. |
| `keywords` | string | No | Comma-separated provenance/history keyword tokens. Supported values: `one_owner`, `clean_title`, `branded`, `fleet`. |

### Dealer Filters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `state` | string | No | Comma-separated two-letter dealer states. |
| `dealer_id` | string | No | Comma-separated dealer UUIDs. Accepts up to 50 dealer IDs. |
| `dealer_type` | string | No | Comma-separated dealer types. |

### Inventory Status Filters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `availability_status` | string | No | Comma-separated availability statuses: `stock`, `transit`, `build`. |
| `inventory_type` | string | No | Comma-separated inventory classes: `new`, `used`, `certified`. `cpo` is accepted as an alias for `certified`. |
| `inventory_status` | enum | No | Inventory mode for facet counts. `active` is the default. Options: `active`, `sold`. |
| `sold_within_days` | string | No | Positive integer day window for sold inventory facets. |
| `snapshot_date` | string | No | Historical active-inventory snapshot date in `YYYY-MM-DD` format. |

### Numeric Range Filters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `min_price` | string | No | Minimum listed price in whole dollars. |
| `max_price` | string | No | Maximum listed price in whole dollars. |
| `min_mileage` | string | No | Minimum odometer mileage. |
| `max_mileage` | string | No | Maximum odometer mileage. |
| `min_msrp` | string | No | Minimum combined MSRP in whole dollars. |
| `max_msrp` | string | No | Maximum combined MSRP in whole dollars. |
| `min_days_on_market` | string | No | Minimum days on market. |
| `max_days_on_market` | string | No | Maximum days on market. |

### Geographic Filters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `latitude` | string | No | Search origin latitude for radius filtering. |
| `longitude` | string | No | Search origin longitude for radius filtering. |
| `postal_code` | string | No | US ZIP/postal search origin for radius filtering. |
| `radius` | string | No | Maximum distance from the search origin, in miles. Requires `postal_code` or `latitude`+`longitude`. Maximum `500`. |
| `bbox` | string | No | Map viewport as `west,south,east,north`. Cannot be combined with `radius`. Maximum diagonal is 1000 miles. |

### Exclusion Filters

| Parameter | Type | Description |
|-----------|------|-------------|
| `exclude_make` | string | Comma-separated makes to exclude. |
| `exclude_model` | string | Comma-separated models to exclude. |
| `exclude_trim` | string | Comma-separated trims to exclude. |
| `exclude_year` | string | Comma-separated model years to exclude. |
| `exclude_state` | string | Comma-separated dealer states to exclude. |
| `exclude_inventory_type` | string | Comma-separated inventory classes to exclude. |
| `exclude_body_type` | string | Comma-separated body types to exclude. |
| `exclude_transmission` | string | Comma-separated transmission values to exclude. |
| `exclude_drivetrain` | string | Comma-separated drivetrain values to exclude. |
| `exclude_version` | string | Comma-separated vehicle versions to exclude. |
| `exclude_engine` | string | Comma-separated engine descriptions to exclude. |
| `exclude_assembly_location` | string | Plus-separated assembly locations to exclude. Uses `+` to match the private API separator. |
| `exclude_assembly_country` | string | Comma-separated assembly countries to exclude. |
| `exclude_exterior_color` | string | Comma-separated exterior colors to exclude. |
| `exclude_interior_color` | string | Comma-separated interior colors to exclude. |
| `exclude_base_exterior_color` | string | Comma-separated normalized exterior colors to exclude. |
| `exclude_base_interior_color` | string | Comma-separated normalized interior colors to exclude. |
| `exclude_options_packages` | string | Comma-separated manufacturer option/package codes to exclude. |
| `exclude_features` | string | Comma-separated feature tokens to exclude. |
| `exclude_fuel_type` | string | Comma-separated fuel types to exclude. |
| `exclude_powertrain_type` | string | Comma-separated powertrain types to exclude. |
| `exclude_keywords` | string | Comma-separated keyword tokens to exclude. |

---

## Response Schema (200)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `data` | object | Yes | Facet counts and numeric ranges for the supplied filters. |
| `data.total` | integer | Yes | Total number of matching listings. |
| `data.facets` | object | Yes | Map of categorical facet name to array of `{ value, count }` (or `{ value, metric }` if a non-count metric is used) bucket objects. |
| `data.range_facets` | object | No | Map of numeric facet name to range bucket data. Each range facet includes `buckets` (array of `{ min, max, count }`), `interval`, `min`, and `max`. |
| `data.stats` | object | No | Map of numeric facet name to descriptive statistics (`min`, `max`, `count`, `missing`, `mean`, `median`, `stddev`). |
| `meta` | object | Yes | Facet response metadata. Includes `facets` (requested facets list), `metric`, `sort`, and `minimum_metric_count`. |

---

## HTTP Status Codes

| Code | Description |
|------|-------------|
| `200` | Successful response |
| `400` | Validation error (bad parameters) |
| `401` | Authentication error |
| `402` | Payment required / quota exceeded |
| `403` | Authorization error |
| `429` | Rate limit exceeded |
| `503` | Service unavailable |
