//! # Carbem
//!
//! A Rust library for retrieving carbon emission values from cloud providers.
//!
//! This library provides a unified interface for querying carbon emission data
//! from various cloud service providers, helping developers build more
//! environmentally conscious applications.

pub mod client;
pub mod error;
pub mod models;
pub mod providers;

// Only expose the client module - all other types are re-exported from there
pub use client::*;
