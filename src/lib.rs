mod error;
mod models;

mod client;
mod pagination;
mod transport;

// Public API surface — callers import only from the crate root.
pub use client::{AsyncVisorClient, ClientConfig, VisorClient};
pub use error::{ApiErrorBody, VisorError};
pub use models::{
    AvailabilityStatus, BBox, CountryCode, DealerAddress, DealerDetail, DealerFilter, DealerRef,
    DealerSummary, DealerType, DealersPage, FacetBucket, FacetField, FacetMetric,
    FacetMetricAggregate, FacetMetricMeasure, FacetSort, FacetsData, FacetsFilter, FacetsMeta,
    FacetsResponse, FieldStats, GeoFilter, GeoOrigin, HistoryKeyword, InventoryModeFilter,
    InventoryType, Latitude, ListingDetail, ListingField, ListingInclude, ListingSnapshot,
    ListingSummary, ListingsFilter, ListingsFilterBase, ListingsPage, Longitude, Pagination,
    PostalCode, PriceHistoryEntry, RadiusMiles, RangeBucket, RangeFacet, SortOrder, StateCode,
    UsageMeta, UsageRecord, UsageSummary, UsageTotals, VehicleBuild, VehicleOption, VehicleRecord,
    VinDetail, VinPattern,
};
