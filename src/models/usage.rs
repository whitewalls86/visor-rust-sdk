use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct UsageRecord {
    pub date: NaiveDate,
    pub metering_class: String,
    pub requests: i64,
    pub charged_micros: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageTotals {
    pub requests: i64,
    pub charged_micros: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageMeta {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub interval: String,
    pub currency: String,
    pub source: String,
    pub freshness: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageSummary {
    pub data: Vec<UsageRecord>,
    pub totals: UsageTotals,
    pub meta: UsageMeta,
}
