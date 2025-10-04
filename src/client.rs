use std::env;

use crate::error::{CarbemError, Result};
use crate::models::{CarbonEmission, EmissionQuery};
use crate::providers::azure::{AzureConfig, AzureProvider};
use crate::providers::CarbonProvider;

/// Main client for querying carbon emissions from cloud providers
#[derive(Debug)]
pub struct CarbemClient {
    azure_provider: Option<AzureProvider>,
}

impl CarbemClient {
    // Create a new empty client
    pub fn new() -> Self {
        Self {
            azure_provider: None,
        }
    }

    // Configure Azure provider
    pub fn with_azure(mut self, config: AzureConfig) -> Result<Self> {
        self.azure_provider = Some(AzureProvider::new(config)?);
        Ok(self)
    }

    // Configure Azure provider from environment variables
    pub fn with_azure_from_env(mut self) -> Result<Self> {
        let access_token = env::var("AZURE_TOKEN")
            .or_else(|_| env::var("CARBEM_AZURE_ACCESS_TOKEN"))
            .map_err(|_| {
                CarbemError::Config(
                    "AZURE_TOKEN or CARBEM_AZURE_ACCESS_TOKEN environment variable not set"
                        .to_string(),
                )
            })?;

        let config = AzureConfig { access_token };
        self.azure_provider = Some(AzureProvider::new(config)?);
        Ok(self)
    }

    // Query carbon emissions
    pub async fn query_emissions(&self, query: &EmissionQuery) -> Result<Vec<CarbonEmission>> {
        match query.provider.as_str() {
            "azure" => {
                let provider = self.azure_provider.as_ref().ok_or_else(|| {
                    CarbemError::Config("Azure provider not configured".to_string())
                })?;
                provider.get_emissions(query).await
            }
            _ => Err(CarbemError::UnsupportedProvider(query.provider.clone())),
        }
    }

    // Check if a provider is configured
    pub fn is_provider_configured(&self, provider: &str) -> bool {
        match provider {
            "azure" => self.azure_provider.is_some(),
            _ => false,
        }
    }
}

impl Default for CarbemClient {
    fn default() -> Self {
        Self::new()
    }
}
