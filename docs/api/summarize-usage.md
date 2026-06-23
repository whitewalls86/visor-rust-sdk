# Summarize Usage

**Endpoint:** `GET /v1/usage`

Returns daily billable usage totals for the authenticated API account. Usage analytics are eventually consistent with the billing ledger and are intended for customer dashboards and reconciliation.

---

## Authorization

| Header | Type | Required | Description |
|--------|------|----------|-------------|
| `Authorization` | string | Yes | Send API keys as `Authorization: Bearer <api_key>`. Query-string API keys are rejected. |

---

## Example Request

```bash
curl --request GET \
  --url https://api.visor.vin/v1/usage \
  --header 'Authorization: Bearer <token>'
```

---

## Example Response (200)

```json
{
  "data": [
    {
      "date": "2026-06-09",
      "metering_class": "listing_search",
      "requests": 1,
      "charged_micros": 2000
    }
  ],
  "totals": {
    "requests": 1,
    "charged_micros": 2000
  },
  "meta": {
    "start_date": "2026-05-11",
    "end_date": "2026-06-09",
    "interval": "day",
    "currency": "USD",
    "source": "public_api_usage_events",
    "freshness": "Usage analytics are eventually consistent with the billing ledger."
  }
}
```

---

## Query Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `start_date` | string | No | Inclusive UTC start date in `YYYY-MM-DD` format. Defaults to 29 days before `end_date`. |
| `end_date` | string | No | Inclusive UTC end date in `YYYY-MM-DD` format. Defaults to today. |
| `metering_class` | string | No | Optional comma-separated metering classes to include. |

---

## Response Schema (200)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `data` | object[] | Yes | Daily usage buckets ordered by date and metering class. |
| `data[].date` | string | Yes | Date of the usage bucket in `YYYY-MM-DD` format. |
| `data[].metering_class` | string | Yes | The billing metering class for this bucket (e.g., `listing_search`). |
| `data[].requests` | integer | Yes | Number of billable API requests for this date and metering class. |
| `data[].charged_micros` | integer | Yes | Billable amount in microcurrency units (divide by 1,000,000 for the value in the stated currency). |
| `totals` | object | Yes | Aggregate totals across the full date range. |
| `totals.requests` | integer | Yes | Total billable requests across the query period. |
| `totals.charged_micros` | integer | Yes | Total charged amount in microcurrency units. |
| `meta` | object | Yes | Response metadata. |
| `meta.start_date` | string | Yes | Actual start date used for the query (`YYYY-MM-DD`). |
| `meta.end_date` | string | Yes | Actual end date used for the query (`YYYY-MM-DD`). |
| `meta.interval` | string | Yes | Aggregation interval (e.g., `day`). |
| `meta.currency` | string | Yes | Currency for charged amounts (e.g., `USD`). |
| `meta.source` | string | Yes | Data source identifier. |
| `meta.freshness` | string | Yes | Human-readable note about data consistency guarantees. |

---

## Notes

- Usage analytics are **eventually consistent** with the billing ledger. Figures are intended for dashboards and reconciliation, not real-time billing verification.
- `charged_micros` is expressed in microcurrency units. For USD, divide by `1,000,000` to get dollar amounts (e.g., `2000` micros = `$0.002`).

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
