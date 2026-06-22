use crate::client::ClientConfig;
use crate::error::VisorError;

/// Sync client for the Visor Public API.
#[derive(Debug)]
pub struct VisorClient {
    #[allow(dead_code)] // used by transport in Phase 3
    pub(crate) config: ClientConfig,
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
        Self { config }
    }
}
