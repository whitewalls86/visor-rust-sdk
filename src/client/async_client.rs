use chrono::NaiveDate;
use uuid::Uuid;

use crate::client::{build_include_params, build_usage_params, validate_listing_id, ClientConfig};
use crate::error::VisorError;
use crate::models::base::ListingInclude;
use crate::models::dealers::{DealerDetail, DealerFilter, DealersPage};
use crate::models::facets::{FacetsFilter, FacetsResponse};
use crate::models::listings::{ListingDetail, ListingsFilter, ListingsPage};
use crate::models::usage::UsageSummary;
use crate::models::vins::{Vin, VinDetail};
use crate::transport::{encode_path_segment, AsyncVisorTransport};

/// Async client for the Visor Public API.
#[derive(Debug)]
pub struct AsyncVisorClient {
    transport: AsyncVisorTransport,
}

impl AsyncVisorClient {
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
            transport: AsyncVisorTransport::new(config),
        }
    }

    // ── Listing methods ───────────────────────────────────────────────────────

    pub async fn filter_listings(
        &self,
        filter: &ListingsFilter,
    ) -> Result<ListingsPage, VisorError> {
        filter.validate()?;
        self.transport.get("/listings", filter.to_params()).await
    }

    pub async fn get_listing(
        &self,
        id: &str,
        include: Option<Vec<ListingInclude>>,
    ) -> Result<ListingDetail, VisorError> {
        validate_listing_id(id)?;
        let params = build_include_params(include);
        self.transport
            .get_one(&format!("/listings/{}", encode_path_segment(id)), params)
            .await
    }

    // ── VIN method ────────────────────────────────────────────────────────────

    pub async fn lookup_vin(
        &self,
        vin: &Vin,
        include: Option<Vec<ListingInclude>>,
    ) -> Result<VinDetail, VisorError> {
        let params = build_include_params(include);
        self.transport
            .get_one(&format!("/vins/{}", vin.as_str()), params)
            .await
    }

    // ── Facets method ─────────────────────────────────────────────────────────

    pub async fn filter_facets(&self, filter: &FacetsFilter) -> Result<FacetsResponse, VisorError> {
        filter.validate()?;
        self.transport.get("/facets", filter.to_params()).await
    }

    // ── Dealer methods ────────────────────────────────────────────────────────

    pub async fn search_dealers(&self, filter: &DealerFilter) -> Result<DealersPage, VisorError> {
        filter.validate()?;
        self.transport.get("/dealers", filter.to_params()).await
    }

    pub async fn get_dealer(&self, id: Uuid) -> Result<DealerDetail, VisorError> {
        self.transport
            .get_one(
                &format!("/dealers/{}", encode_path_segment(&id.to_string())),
                vec![],
            )
            .await
    }

    pub async fn dealer_inventory(
        &self,
        dealer_id: Uuid,
        filter: &ListingsFilter,
    ) -> Result<ListingsPage, VisorError> {
        filter.validate()?;
        let path = format!(
            "/dealers/{}/listings",
            encode_path_segment(&dealer_id.to_string())
        );
        self.transport.get(&path, filter.to_params()).await
    }

    // ── Usage method ──────────────────────────────────────────────────────────

    pub async fn get_usage(
        &self,
        start: Option<NaiveDate>,
        end: Option<NaiveDate>,
        metering_class: Option<Vec<String>>,
    ) -> Result<UsageSummary, VisorError> {
        let params = build_usage_params(start, end, metering_class)?;
        self.transport.get("/usage", params).await
    }
}
