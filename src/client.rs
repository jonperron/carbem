use crate::{Result, CarbonEmission, EmissionQuery};
use reqwest::Client;
use std::time::Duration;

/// The main client for interacting with carbon emission data providers
#[derive(Debug, Clone)]
pub struct CarbemClient {
    http_client: Client,
}

impl CarbemClient {
    /// Create a new CarbemClient with default configuration
    pub fn new() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { http_client }
    }
    
    /// Create a new CarbemClient with custom HTTP client
    pub fn with_client(http_client: Client) -> Self {
        Self { http_client }
    }
    
    /// Query carbon emissions from a specific provider
    pub async fn get_emissions(&self, query: EmissionQuery) -> Result<Vec<CarbonEmission>> {
        // This method will be implemented when specific providers are added
        todo!("Implementation will be added when providers are implemented")
    }
    
    /// Get available providers
    pub fn get_supported_providers(&self) -> Vec<&'static str> {
        // Will be updated as providers are implemented
        vec![]
    }
    
    /// Get available regions for a provider
    pub async fn get_regions(&self, provider: &str) -> Result<Vec<String>> {
        // This method will be implemented when specific providers are added
        todo!("Implementation will be added when providers are implemented")
    }
}

impl Default for CarbemClient {
    fn default() -> Self {
        Self::new()
    }
}