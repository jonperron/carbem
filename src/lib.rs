//! # Carbem
//!
//! A Rust library for retrieving carbon emission values from cloud providers.
//!
//! This library provides both a native Rust API and an FFI layer for use in other languages.
//!
//! ## Rust API (Recommended for Rust applications)
//!
//! ```rust,no_run
//! use carbem::{CarbemClient, EmissionQuery, TimePeriod, AzureConfig};
//! use chrono::Utc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = CarbemClient::builder()
//!         .with_azure_from_env()?
//!         .build();
//!
//!     let query = EmissionQuery {
//!         provider: "azure".to_string(),
//!         regions: vec!["subscription-id".to_string()],
//!         time_period: TimePeriod {
//!             start: Utc::now() - chrono::Duration::days(30),
//!             end: Utc::now(),
//!         },
//!         services: None,
//!         resources: None,
//!     };
//!
//!     let emissions = client.query_emissions(&query).await?;
//!     println!("Found {} emissions", emissions.len());
//!     Ok(())
//! }
//! ```
//!
//! ## FFI API (For Python/TypeScript bindings)
//!
//! ```rust,no_run
//! use carbem::get_emissions;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = r#"{"access_token": "your-token"}"#;
//!     let payload = r#"{"start_date": "2024-01-01T00:00:00Z", "end_date": "2024-02-01T00:00:00Z", "regions": ["sub-id"], "services": null, "resources": null}"#;
//!     let emissions = get_emissions("azure", config, payload).await?;
//!     Ok(())
//! }
//! ```
use pyo3::prelude::*;
use pyo3::types::PyModule;

pub mod client;
pub mod error;
pub mod ffi;
pub mod models;
pub mod providers;

// Export the main Rust API
pub use client::*;

// Export core types
pub use error::{CarbemError, Result};
pub use models::{CarbonEmission, EmissionMetadata, EmissionQuery, TimePeriod};
pub use providers::azure::{AzureConfig, AzureProvider};

// Export FFI functions for Python/TS bindings
pub use ffi::get_emissions;

/// Get carbon emissions from cloud providers (Python-compatible function)
#[pyfunction]
pub fn get_emissions_py(provider: &str, config_json: &str, query_json: &str) -> PyResult<String> {
    // Use the existing FFI function with runtime block
    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create runtime: {}",
            e
        ))
    })?;

    match rt.block_on(get_emissions(provider, config_json, query_json)) {
        Ok(emissions) => {
            // Convert emissions to JSON string
            serde_json::to_string(&emissions).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Serialization error: {}",
                    e
                ))
            })
        }
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "{}",
            e
        ))),
    }
}

/// Python module
#[pymodule]
fn carbem(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_emissions_py, m)?)?;
    Ok(())
}
