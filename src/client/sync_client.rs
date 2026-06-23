use chrono::NaiveDate;

use crate::client::ClientConfig;
use crate::error::VisorError;
use crate::models::base::ListingInclude;
use crate::models::dealers::{DealerDetail, DealerFilter, DealersPage};
use crate::models::facets::{FacetsFilter, FacetsResponse};
use crate::models::listings::{ListingDetail, ListingsFilter, ListingsPage};
use crate::models::usage::UsageSummary;
use crate::models::vins::VinDetail;
use crate::transport::SyncVisorTransport;

/// Sync client for the Visor Public API.
#[derive(Debug)]
pub struct VisorClient {
    transport: SyncVisorTransport,
}

impl VisorClient {
    /// Create a client with an explicit API key and default configuration.
    ///
    /// Panics if `api_key` is empty.
    pub fn new(api_key: String) -> Self {
        Self::with_config(ClientConfig {
            api_key,
            ..ClientConfig::default()
        })
    }

    /// Create a client from the `VISOR_API_KEY` environment variable.
    pub fn from_env() -> Result<Self, VisorError> {
        let key = std::env::var("VISOR_API_KEY").map_err(|_| VisorError::MissingApiKey)?;
        Ok(Self::new(key))
    }

    /// Create a client with full configuration control.
    ///
    /// Panics if `config.api_key` is empty.
    pub fn with_config(config: ClientConfig) -> Self {
        assert!(!config.api_key.is_empty(), "api_key must not be empty");
        Self {
            transport: SyncVisorTransport::new(config),
        }
    }

    // ── Implemented methods ───────────────────────────────────────────────────

    pub fn filter_listings(&self, filter: &ListingsFilter) -> Result<ListingsPage, VisorError> {
        filter.validate()?;
        self.transport.get("/listings", filter.to_params())
    }

    pub fn get_listing(
        &self,
        id: &str,
        _include: Option<Vec<ListingInclude>>,
    ) -> Result<ListingDetail, VisorError> {
        // Phase 4 TODO: serialize include params via ListingInclude::as_str()
        self.transport.get(&format!("/listings/{id}"), vec![])
    }

    // ── Stub methods — Phase 5 TODO ───────────────────────────────────────────

    pub fn lookup_vin(
        &self,
        _vin: &str,
        _include: Option<Vec<ListingInclude>>,
    ) -> Result<VinDetail, VisorError> {
        Err(VisorError::InvalidResponse {
            message: "not implemented".to_string(),
        })
    }

    pub fn filter_facets(&self, _filter: &FacetsFilter) -> Result<FacetsResponse, VisorError> {
        Err(VisorError::InvalidResponse {
            message: "not implemented".to_string(),
        })
    }

    pub fn search_dealers(&self, _filter: &DealerFilter) -> Result<DealersPage, VisorError> {
        Err(VisorError::InvalidResponse {
            message: "not implemented".to_string(),
        })
    }

    pub fn get_dealer(&self, _id: &str) -> Result<DealerDetail, VisorError> {
        Err(VisorError::InvalidResponse {
            message: "not implemented".to_string(),
        })
    }

    pub fn dealer_inventory(
        &self,
        _dealer_id: &str,
        _filter: &ListingsFilter,
    ) -> Result<ListingsPage, VisorError> {
        Err(VisorError::InvalidResponse {
            message: "not implemented".to_string(),
        })
    }

    pub fn get_usage(
        &self,
        _start: Option<NaiveDate>,
        _end: Option<NaiveDate>,
        _metering_class: Option<Vec<String>>,
    ) -> Result<UsageSummary, VisorError> {
        Err(VisorError::InvalidResponse {
            message: "not implemented".to_string(),
        })
    }
}
