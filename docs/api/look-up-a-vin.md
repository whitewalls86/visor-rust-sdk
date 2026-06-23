# Look Up a VIN

**Endpoint:** `GET /v1/vins/{vin}`

Returns the current or latest known record for a VIN. Unknown VINs return `404 not_found_error`; known VINs can return `status: missing`.

---

## Authorization

| Header | Type | Required | Description |
|--------|------|----------|-------------|
| `Authorization` | string | Yes | Send API keys as `Authorization: Bearer <api_key>`. Query-string API keys are rejected. |

---

## Example Request

```bash
curl --request GET \
  --url https://api.visor.vin/v1/vins/{vin} \
  --header 'Authorization: Bearer <token>'
```

---

## Example Response (200)

```json
{
  "data": {
    "vin": "4T1DAACKXTU765422",
    "status": "active",
    "build": {
      "year": 2026,
      "make": "Toyota",
      "model": "Camry",
      "trim": "SE",
      "version": "SE Hybrid",
      "body_type": "Sedan",
      "drivetrain": "FWD",
      "fuel_type": "Hybrid",
      "powertrain_type": "HEV",
      "transmission": "CVT",
      "engine": "2.5L I4",
      "cylinders": 4,
      "doors": 4,
      "seating_capacity": 5,
      "exterior_color": "Dark Cosmos",
      "interior_color": "Black SofTex",
      "base_exterior_color": "Blue",
      "base_interior_color": "Black",
      "assembly_location": null,
      "window_sticker_verified": false,
      "base_msrp": 31800,
      "combined_msrp": 35711,
      "options": [
        {
          "code": "3J",
          "name": "Blackout Emblem Overlays",
          "msrp": 89
        },
        {
          "code": "2T",
          "name": "All-Weather Floor Liner Package",
          "msrp": 319
        }
      ]
    },
    "latest_listing": {
      "id": "0043554b54709f18a7bcf42f23e5e6ef",
      "price": 35236,
      "miles": 0,
      "inventory_type": "new",
      "stock_number": "263929",
      "vdp_url": "https://www.northhollywoodtoyota.com/viewdetails/new/4t1daackxtu765422",
      "vhr_url": null,
      "photo_urls": [
        "https://delivery.via.assetscs.toyota.com/adobe/assets/urn:aaid:aem:260ec892-e9be-4815-9148-8c139edd5b95/as/image.png?fmt=png-alpha%2Crgb%2Cnone"
      ],
      "photo_url_primary": "https://delivery.via.assetscs.toyota.com/adobe/assets/urn:aaid:aem:260ec892-e9be-4815-9148-8c139edd5b95/as/image.png?fmt=png-alpha%2Crgb%2Cnone",
      "inventory_date": "2026-06-09",
      "sold_date": null,
      "last_checked_at": "2026-06-09 11:40:57.534384",
      "dealer": {
        "dealer_id": "16ed7612-0ffd-4e1e-88db-174f8dd57c54",
        "name": "North Hollywood Toyota",
        "city": "North Hollywood",
        "state": "CA",
        "latitude": 34.154,
        "longitude": -118.3678,
        "phone": "(818) 369-3922"
      },
      "price_history": []
    }
  },
  "meta": {}
}
```

---

## Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `vin` | string | **Yes** | 17-character VIN. I, O, and Q are not valid VIN characters. |

---

## Query Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `include` | string | No | Comma-separated optional expansions. Supported values: `price_history`, `options`. |

---

## Response Schema (200)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `data` | object | Yes | VIN detail record. |
| `data.vin` | string | Yes | 17-character VIN. |
| `data.status` | string | Yes | VIN status. Possible values: `active`, `missing`. Known VINs with no current listing return `missing`. |
| `data.build` | object | Yes | Vehicle build specification. Includes `year`, `make`, `model`, `trim`, `version`, `body_type`, `drivetrain`, `fuel_type`, `powertrain_type`, `transmission`, `engine`, `cylinders`, `doors`, `seating_capacity`, `exterior_color`, `interior_color`, `base_exterior_color`, `base_interior_color`, `assembly_location`, `window_sticker_verified`, `base_msrp`, `combined_msrp`, and `options` array. |
| `data.build.options` | object[] | Yes | Array of option/package objects. Each has `code`, `name`, and `msrp`. |
| `data.latest_listing` | object/null | Yes | Most recent listing associated with this VIN, or `null` if none. Contains listing fields: `id`, `price`, `miles`, `inventory_type`, `stock_number`, `vdp_url`, `vhr_url`, `photo_urls`, `photo_url_primary`, `inventory_date`, `sold_date`, `last_checked_at`, `dealer`, and `price_history`. |
| `data.latest_listing.dealer` | object | Yes | Dealer summary. Fields: `dealer_id`, `name`, `city`, `state`, `latitude`, `longitude`, `phone`. |
| `meta` | object | Yes | Reserved metadata object. Empty for the beta contract unless an endpoint documents otherwise. |

---

## Notes

- Unknown VINs return `404 not_found_error`.
- Known VINs that are no longer listed anywhere return `status: missing` (200 response, not 404).

---

## HTTP Status Codes

| Code | Description |
|------|-------------|
| `200` | Successful response |
| `400` | Validation error (bad parameters) |
| `401` | Authentication error |
| `402` | Payment required / quota exceeded |
| `403` | Authorization error |
| `404` | VIN not found |
| `429` | Rate limit exceeded |
| `503` | Service unavailable |
