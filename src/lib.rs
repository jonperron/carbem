//! # Carbem
//! 
//! A Rust library for retrieving carbon emission values from cloud providers.
//! 
//! This library provides a unified interface for querying carbon emission data
//! from various cloud service providers, helping developers build more
//! environmentally conscious applications.

pub mod error;
pub mod models;
pub mod providers;
pub mod client;

pub use error::{CarbemError, Result};
pub use models::*;
pub use client::CarbemClient;

/// The main entry point for the Carbem library.
/// 
/// # Examples
/// 
/// ```rust
/// use carbem::CarbemClient;
/// 
/// #[tokio::main]
/// async fn main() -> carbem::Result<()> {
///     let client = CarbemClient::new();
///     // Usage examples will be added as providers are implemented
///     Ok(())
/// }
