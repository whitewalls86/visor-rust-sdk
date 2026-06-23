# Get a Listing

**Endpoint:** `GET /v1/listings/{listing_id}`

Returns listing-centered detail by `listing_id`. Missing listings return `404 not_found_error`.

---

## Authorization

| Header | Type | Required | Description |
|--------|------|----------|-------------|
| `Authorization` | string | Yes | Send API keys as `Authorization: Bearer <api_key>`. Query-string API keys are rejected. |

---

## Example Request

```bash
curl --request GET \
  --url https://api.visor.vin/v1/listings/{listing_id} \
  --header 'Authorization: Bearer <token>'
```

---

## Example Response (200)

```json
{
  "data": {
    "id": "0043554b54709f18a7bcf42f23e5e6ef",
    "vin": "4T1DAACKXTU765422",
    "status": "active",
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
    "vehicle": {
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
      }
    },
    "price_history": []
  },
  "meta": {}
}
```

---

## Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `listing_id` | string | **Yes** | Stable listing identifier. |

---

## Query Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `include` | string | No | Comma-separated optional expansions. Supported values: `price_history`, `options`. |

---

## Response Schema (200)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `data` | object | Yes | Listing detail record. |
| `data.id` | string | Yes | Stable listing identifier. |
| `data.vin` | string | Yes | 17-character VIN. |
| `data.status` | string | Yes | Listing status (e.g., `active`). |
| `data.price` | integer/null | Yes | Listed price in dollars. |
| `data.miles` | integer | Yes | Odometer reading. |
| `data.inventory_type` | string | Yes | Inventory class: `new`, `used`, or `certified`. |
| `data.stock_number` | string/null | Yes | Dealer stock number. |
| `data.vdp_url` | string/null | Yes | Vehicle detail page URL at the dealer's website. |
| `data.vhr_url` | string/null | Yes | Vehicle history report URL. |
| `data.photo_urls` | string[] | Yes | Array of photo URLs. |
| `data.photo_url_primary` | string/null | Yes | Primary photo URL. |
| `data.inventory_date` | string | Yes | Date the listing first appeared (`YYYY-MM-DD`). |
| `data.sold_date` | string/null | Yes | Date the listing was marked sold (`YYYY-MM-DD`), or `null`. |
| `data.last_checked_at` | string | Yes | Timestamp of the most recent data refresh. |
| `data.dealer` | object | Yes | Dealer summary embedded in the listing. Fields: `dealer_id`, `name`, `city`, `state`, `latitude`, `longitude`, `phone`. |
| `data.vehicle` | object | Yes | VIN-level vehicle record. Contains `vin`, `status`, and a `build` object with full spec fields and `options` array. |
| `data.price_history` | object[] | Yes | Price change events (populated when `include=price_history`). |
| `meta` | object | Yes | Reserved metadata object. Empty for the beta contract unless an endpoint documents otherwise. |

---

## HTTP Status Codes

| Code | Description |
|------|-------------|
| `200` | Successful response |
| `400` | Validation error (bad parameters) |
| `401` | Authentication error |
| `402` | Payment required / quota exceeded |
| `403` | Authorization error |
| `404` | Listing not found |
| `429` | Rate limit exceeded |
| `503` | Service unavailable |
