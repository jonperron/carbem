//! Cloud provider modules for carbon emission data retrieval

pub mod aws;
pub mod azure;
pub mod gcp;

use crate::{Result, CarbonEmission, EmissionQuery};
use async_trait::async_trait;

/// Trait that all carbon emission providers must implement
#[async_trait]
pub trait CarbonProvider {
    /// Get the provider name
    fn name(&self) -> &'static str;
    
    /// Get supported regions for this provider
    async fn get_regions(&self) -> Result<Vec<String>>;
    
    /// Query carbon emissions for the given parameters
    async fn get_emissions(&self, query: &EmissionQuery) -> Result<Vec<CarbonEmission>>;
    
    /// Check if the provider is properly configured
    fn is_configured(&self) -> bool;
}