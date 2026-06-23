use serde::de::DeserializeOwned;

use crate::client::ClientConfig;
use crate::error::VisorError;

#[derive(Debug)]
pub(crate) struct SyncVisorTransport {
    client: reqwest::blocking::Client,
    config: ClientConfig,
}

impl SyncVisorTransport {
    pub(crate) fn new(config: ClientConfig) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("failed to build reqwest blocking client");
        Self { client, config }
    }

    /// Fetch a single-resource endpoint whose response is `{ "data": T, "meta": {} }`.
    pub(crate) fn get_one<T: DeserializeOwned>(
        &self,
        path: &str,
        params: Vec<(String, String)>,
    ) -> Result<T, VisorError> {
        let envelope: super::DataEnvelope<T> = self.get(path, params)?;
        Ok(envelope.data)
    }

    pub(crate) fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: Vec<(String, String)>,
    ) -> Result<T, VisorError> {
        let url = super::build_url_with_params(&self.config.base_url, path, &params);
        let response = self
            .client
            .get(url)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.config.api_key),
            )
            .send()
            .map_err(VisorError::TransportError)?;

        let status = response.status();
        let status_u16 = status.as_u16();

        if status.is_success() {
            let bytes = response.bytes().map_err(VisorError::TransportError)?;
            serde_json::from_slice(&bytes).map_err(|e| VisorError::InvalidResponse {
                message: e.to_string(),
            })
        } else {
            let retry_after = if status_u16 == 429 {
                super::parse_retry_after(response.headers())
            } else {
                None
            };
            let bytes = response.bytes().map_err(VisorError::TransportError)?;
            let body = super::parse_error_body(status_u16, &bytes);
            Err(super::map_status_to_error(status_u16, body, retry_after))
        }
    }
}
