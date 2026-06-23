# Search Dealers

**Endpoint:** `GET /v1/dealers`

Returns public dealer summaries ordered by active listing count. Listing counts use VIN-level listing deduplication and ownership assignment, so they can differ from raw dealer feed counts. Use `dealer_id`, `state`, `country`, `type`, `make`, and `q` to narrow results.

---

## Authorization

| Header | Type | Required | Description |
|--------|------|----------|-------------|
| `Authorization` | string | Yes | Send API keys as `Authorization: Bearer <api_key>`. Query-string API keys are rejected. |

---

## Example Request

```bash
curl --request GET \
  --url https://api.visor.vin/v1/dealers \
  --header 'Authorization: Bearer <token>'
```

---

## Example Response (200)

```json
{
  "data": [
    {
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
      "listing_count": 1656
    }
  ],
  "pagination": {
    "limit": 2,
    "offset": 0,
    "total": 130,
    "next_offset": 2
  },
  "meta": {}
}
```

---

## Query Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `limit` | string | No | Page size as an integer string. Defaults to `50`; maximum `100`. |
| `offset` | string | No | Zero-based page offset as an integer string. Defaults to `0`. |
| `dealer_id` | string | No | Comma-separated dealer UUIDs to fetch directly. Accepts up to 100 dealer IDs. |
| `state` | string | No | Comma-separated two-letter dealer states, e.g., `CA,TX`. |
| `country` | string | No | Dealer country code, e.g., `US`. |
| `type` | enum | No | Dealer type. Options: `franchise`, `independent`. |
| `make` | string | No | Comma-separated represented franchise makes or slugs, e.g., `toyota,honda`. |
| `q` | string | No | Case-insensitive dealer name or website domain search string. |

---

## Response Schema (200)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `data` | object[] | Yes | Array of dealer summary records. |
| `data[].dealer_id` | string | Yes | Dealer UUID. |
| `data[].name` | string | Yes | Dealer name. |
| `data[].city` | string | Yes | City where the dealer is located. |
| `data[].state` | string | Yes | Two-letter state code. |
| `data[].country` | string | Yes | Country code (e.g., `US`). |
| `data[].latitude` | number | Yes | Dealer latitude. |
| `data[].longitude` | number | Yes | Dealer longitude. |
| `data[].type` | string | Yes | Dealer type: `franchise` or `independent`. |
| `data[].website` | string/null | Yes | Dealer website URL. |
| `data[].makes` | string[] | Yes | Array of franchise make names represented by this dealer. |
| `data[].listing_count` | integer | Yes | Number of active listings attributed to this dealer (after VIN deduplication). |
| `pagination` | object | Yes | Offset pagination metadata. Fields: `limit`, `offset`, `total`, `next_offset`. |
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
| `429` | Rate limit exceeded |
| `503` | Service unavailable |
