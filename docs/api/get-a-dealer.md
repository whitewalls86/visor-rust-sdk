# Get a Dealer

**Endpoint:** `GET /v1/dealers/{dealer_id}`

Returns one dealer summary by `dealer_id`. Missing dealers return `404 not_found_error`.

---

## Authorization

| Header | Type | Required | Description |
|--------|------|----------|-------------|
| `Authorization` | string | Yes | Send API keys as `Authorization: Bearer <api_key>`. Query-string API keys are rejected. |

---

## Example Request

```bash
curl --request GET \
  --url https://api.visor.vin/v1/dealers/{dealer_id} \
  --header 'Authorization: Bearer <token>'
```

---

## Example Response (200)

```json
{
  "data": {
    "dealer_id": "b62c6042-b3a0-4a58-bc5b-55966bd1c68c",
    "name": "Claremont Toyota",
    "city": "Claremont",
    "state": "CA",
    "country": "US",
    "latitude": 34.080929,
    "longitude": -117.72527,
    "type": "franchise",
    "website": "https://claremonttoyota.com",
    "makes": [
      "Toyota"
    ],
    "listing_count": 1656,
    "phone": "(909) 455-9142",
    "address": {
      "line1": "601 Auto Center Dr",
      "city": "Claremont",
      "state": "CA",
      "country": "US"
    }
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

## Response Schema (200)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `data` | object | Yes | Dealer detail record. |
| `data.dealer_id` | string | Yes | Dealer UUID. |
| `data.name` | string | Yes | Dealer name. |
| `data.city` | string | Yes | City where the dealer is located. |
| `data.state` | string | Yes | Two-letter state code. |
| `data.country` | string | Yes | Country code (e.g., `US`). |
| `data.latitude` | number | Yes | Dealer latitude. |
| `data.longitude` | number | Yes | Dealer longitude. |
| `data.type` | string | Yes | Dealer type: `franchise` or `independent`. |
| `data.website` | string/null | Yes | Dealer website URL. |
| `data.makes` | string[] | Yes | Array of franchise make names represented by this dealer. |
| `data.listing_count` | integer | Yes | Number of active listings attributed to this dealer (after VIN deduplication). |
| `data.phone` | string/null | Yes | Dealer phone number. |
| `data.address` | object/null | Yes | Dealer physical address. Fields: `line1`, `city`, `state`, `country`. |
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
| `404` | Dealer not found |
| `429` | Rate limit exceeded |
| `503` | Service unavailable |
