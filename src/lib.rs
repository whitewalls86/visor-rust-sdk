mod error;
mod models;

mod client;
mod pagination;
mod transport;

// Public API surface — callers import only from the crate root.
pub use client::{AsyncVisorClient, ClientConfig, VisorClient};
pub use error::{ApiErrorBody, VisorError};
pub use models::{
    BBox, DealerAddress, DealerDetail, DealerFilter, DealerRef, DealerSummary, DealerType,
    DealersPage, FacetBucket, FacetSort, FacetsData, FacetsFilter, FacetsMeta, FacetsResponse,
    FieldStats, InventoryStatus, ListingDetail, ListingInclude, ListingSnapshot, ListingSummary,
    ListingsFilter, ListingsFilterBase, ListingsPage, Pagination, PriceHistoryEntry, RangeBucket,
    RangeFacet, SortOrder, UsageMeta, UsageRecord, UsageSummary, UsageTotals, VehicleBuild,
    VehicleOption, VehicleRecord, VinDetail,
};
