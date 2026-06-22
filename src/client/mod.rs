pub mod async_client;
pub mod sync_client;

pub use async_client::AsyncVisorClient;
pub use sync_client::VisorClient;

/// Configuration shared by both async and sync clients.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub api_key: String,
    /// Base URL for all requests. Defaults to `https://api.visor.vin/v1`.
    pub base_url: String,
    /// Per-request timeout. Defaults to 30 seconds.
    pub timeout: std::time::Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.visor.vin/v1".to_string(),
            timeout: std::time::Duration::from_secs(30),
        }
    }
}
