# List a Dealer's Inventory

**Endpoint:** `GET /v1/dealers/{dealer_id}/listings`

Returns listing summaries scoped to one dealer after VIN-level listing deduplication and ownership assignment. If a dealer feed contains a VIN that Freeway attributes to another dealer, that VIN may not appear here. Supports the listing filter and pagination surface.

---

## Authorization

| Header | Type | Required | Description |
|--------|------|----------|-------------|
| `Authorization` | string | Yes | Send API keys as `Authorization: Bearer <api_key>`. Query-string API keys are rejected. |

---

## Example Request

```bash
curl --request GET \
  --url https://api.visor.vin/v1/dealers/{dealer_id}/listings \
  --header 'Authorization: Bearer <token>'
```

---

## Example Response (200)

```json
{
  "data": [
    {
      "id": "03738526521c9c73b36cbc2fc28f8891",
      "vin": "3TMLB5JN3TM286572",
      "year": 2026,
      "make": "Toyota",
      "model": "Tacoma",
      "trim": "SR5",
      "price": 41903,
      "miles": 0,
      "status": "active",
      "inventory_status": "active",
      "inventory_type": "new",
      "dealer_id": "b62c6042-b3a0-4a58-bc5b-55966bd1c68c",
      "dealer_name": "Claremont Toyota",
      "city": "Claremont",
      "state": "CA",
      "vdp_url": "https://www.claremonttoyota.com/viewdetails/new/3tmlb5jn3tm286572"
    }
  ],
  "pagination": {
    "limit": 2,
    "offset": 0,
    "total": 1656,
    "next_offset": 2
  },
  "meta": {}
}
```

---

## Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `dealer_id` | string | **Yes** | Dealer UUID. |

---

## Query Parameters

### Pagination & Sorting

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `limit` | string | No | Page size as an integer string. Defaults to `50`; maximum `100`. |
| `offset` | string | No | Zero-based page offset as an integer string. Defaults to `0`. |
| `sort` | enum | No | Sort order. Use field names with optional `-` for descending (e.g., `price` or `-price`). Defaults to `days_on_market` (newest listings first). `discount` returns best discount from MSRP first. `distance` requires `postal_code` or `latitude`+`longitude`. Options: `days_on_market`, `-days_on_market`, `price`, `-price`, `miles`, `-miles`, `msrp`, `-msrp`, `discount`, `-discount`, `distance`. |

### Response Projection

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `fields` | string | No | Comma-separated listing summary fields to return. `id` and `vin` are always returned. Use `default` to include the default fieldset plus selected extras. Unknown fields are rejected. Supported fields: `default`, `id`, `vin`, `year`, `make`, `model`, `trim`, `version`, `body_type`, `drivetrain`, `fuel_type`, `powertrain_type`, `transmission`, `engine`, `cylinders`, `doors`, `seating_capacity`, `exterior_color`, `interior_color`, `base_exterior_color`, `base_interior_color`, `msrp`, `discount_from_msrp`, `price`, `miles`, `days_on_market`, `status`, `inventory_status`, `inventory_type`, `stock_number`, `vdp_url`, `sold_date`, `dealer_id`, `dealer_name`, `dealer_type`, `city`, `state`, `latitude`, `longitude`, `distance_miles`, `photo_urls`, `features`, `options_packages`. |
| `include` | string | No | Comma-separated optional expansions. Supported values: `price_history`, `options`. |

### Vehicle Filters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `make` | string | No | Comma-separated make names or slugs, e.g., `toyota,honda`. |
| `model` | string | No | Comma-separated model names or slugs. Combine with `make` when possible for narrower results. |
| `trim` | string | No | Comma-separated trim names. |
| `year` | string | No | Comma-separated model years, e.g., `2023,2024`. |
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
| `keywords` | string | No | Comma-separated provenance/history keyword tokens. Supported values: `one_owner`, `clean_title`, `branded`, `fleet`. Positive tokens must be present; negative history tokens are excluded. |

### Dealer Filters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `state` | string | No | Comma-separated two-letter dealer states, e.g., `CA,TX`. |
| `dealer_id` | string | No | Comma-separated dealer UUIDs. Accepts up to 50 dealer IDs; uses dealer-filtered listing metering. |
| `dealer_type` | string | No | Comma-separated dealer types, e.g., `franchise,independent`. |

### Inventory Status Filters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `availability_status` | string | No | Comma-separated availability statuses: `stock`, `transit`, `build`. |
| `inventory_type` | string | No | Comma-separated inventory classes: `new`, `used`, `certified`. `cpo` is accepted as an alias for `certified`. |
| `inventory_status` | enum | No | Inventory mode. `active` is the default; `sold` searches historical sold inventory. Options: `active`, `sold`. |
| `sold_within_days` | string | No | Positive integer day window for sold inventory, e.g., `30`. Cannot be combined with `snapshot_date`. |
| `snapshot_date` | string | No | Historical active-inventory snapshot date in `YYYY-MM-DD` format. Cannot be combined with sold inventory filters. |

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
| `latitude` | string | No | Search origin latitude for `sort=distance`, `distance_miles`, and radius filtering. |
| `longitude` | string | No | Search origin longitude for `sort=distance`, `distance_miles`, and radius filtering. |
| `postal_code` | string | No | US ZIP/postal search origin for `sort=distance`, `distance_miles`, and radius filtering. |
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
| `exclude_inventory_type` | string | Comma-separated inventory classes to exclude. `cpo` is accepted as an alias for `certified`. |
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
| `data` | object[] | Yes | Records returned for this page. Each object is a listing summary. |
| `pagination` | object | Yes | Offset pagination metadata. |
| `pagination.limit` | integer | Yes | Page size used for this response. |
| `pagination.offset` | integer | Yes | Zero-based offset used. |
| `pagination.total` | integer | Yes | Total matching records for this dealer. |
| `pagination.next_offset` | integer/null | Yes | Offset to use for the next page, or `null` if on the last page. |
| `meta` | object | Yes | Reserved metadata object. Empty for the beta contract unless an endpoint documents otherwise. |

---

## Notes

- Listing counts may differ from raw dealer feed counts because Freeway applies VIN-level deduplication and ownership assignment. A VIN attributed to another dealer will not appear in this endpoint's results even if that dealer's feed includes it.

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
