use std::time::Duration;

/// Normalized error shape extracted from non-2xx API responses.
///
/// The API returns `{"error": {"code": "...", "message": "..."}}`.
/// The transport extracts those fields plus the HTTP status code.
#[derive(Debug)]
pub struct ApiErrorBody {
    pub status: u16,
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for ApiErrorBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} (HTTP {})", self.code, self.message, self.status)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VisorError {
    /// `VISOR_API_KEY` env var not set and no key passed explicitly.
    #[error("Missing API key: set VISOR_API_KEY or pass key to new()")]
    MissingApiKey,

    /// Filter failed local validation before any HTTP request was made.
    #[error("Invalid filter: {message}")]
    InvalidFilter { message: String },

    /// HTTP 400 — request was well-formed but failed API validation.
    #[error("Validation error: {0}")]
    ValidationError(ApiErrorBody),

    /// HTTP 401 — bad or missing API key.
    #[error("Unauthorized: {0}")]
    AuthError(ApiErrorBody),

    /// HTTP 402 — account requires payment.
    #[error("Payment required: {0}")]
    PaymentRequiredError(ApiErrorBody),

    /// HTTP 403 — key is valid but lacks permission.
    #[error("Forbidden: {0}")]
    ForbiddenError(ApiErrorBody),

    /// HTTP 404 — resource does not exist.
    #[error("Not found: {0}")]
    NotFoundError(ApiErrorBody),

    /// HTTP 429 — rate limited. `retry_after` parsed from the `Retry-After` header.
    #[error("Rate limited {body}; retry after {retry_after:?}")]
    RateLimitError {
        body: ApiErrorBody,
        retry_after: Option<Duration>,
    },

    /// Any other non-2xx response.
    #[error("API error: {0}")]
    VisorApiError(ApiErrorBody),

    /// Network or connection failure (DNS, timeout, TLS, etc.).
    #[error("Transport error: {0}")]
    TransportError(#[from] reqwest::Error),

    /// Successful HTTP response whose body could not be parsed or had an unexpected shape.
    #[error("Invalid response: {message}")]
    InvalidResponse { message: String },
}
