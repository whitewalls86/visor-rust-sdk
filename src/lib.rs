mod error;
mod models;

mod client;
mod pagination;
mod transport;

// Public API surface — callers import only from the crate root.
pub use client::{AsyncVisorClient, ClientConfig, VisorClient};
pub use error::{ApiErrorBody, VisorError};
