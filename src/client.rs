//! Type-safe builder pattern for CarbemClient

use crate::error::{CarbemError, Result};
use crate::models::{CarbonEmission, EmissionQuery};
use crate::providers::azure::AzureConfig;
use crate::providers::registry::ProviderRegistry;
use crate::providers::CarbonProvider;
use serde_json::json;
use std::marker::PhantomData;

/// Type-safe builder for CarbemClient
pub struct CarbemClientBuilder<State> {
    registry: ProviderRegistry,
    providers: Vec<Box<dyn CarbonProvider + Send + Sync>>,
    _state: PhantomData<State>,
}

/// Builder state: No providers configured
pub struct Empty;

/// Builder state: At least one provider configured
pub struct Configured;

impl CarbemClientBuilder<Empty> {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            registry: ProviderRegistry::new(),
            providers: Vec::new(),
            _state: PhantomData,
        }
    }

    /// Add Azure provider with explicit config
    pub fn with_azure(mut self, config: AzureConfig) -> Result<CarbemClientBuilder<Configured>> {
        let provider = self.registry.create_provider("azure", json!(config))?;
        self.providers.push(provider);

        Ok(CarbemClientBuilder {
            registry: self.registry,
            providers: self.providers,
            _state: PhantomData,
        })
    }

    /// Add Azure provider from environment
    pub fn with_azure_from_env(self) -> Result<CarbemClientBuilder<Configured>> {
        let access_token = std::env::var("AZURE_TOKEN")
            .or_else(|_| std::env::var("CARBEM_AZURE_ACCESS_TOKEN"))
            .map_err(|_| {
                CarbemError::Config(
                    "AZURE_TOKEN or CARBEM_AZURE_ACCESS_TOKEN environment variable not set"
                        .to_string(),
                )
            })?;

        let config = AzureConfig { access_token };
        self.with_azure(config)
    }
}

impl CarbemClientBuilder<Configured> {
    /// Add another Azure provider (for multiple subscriptions)
    pub fn with_azure(mut self, config: AzureConfig) -> Result<Self> {
        let provider = self.registry.create_provider("azure", json!(config))?;
        self.providers.push(provider);
        Ok(self)
    }

    /// Add provider from JSON config
    pub fn with_provider_from_json(
        mut self,
        provider_name: &str,
        config_json: &str,
    ) -> Result<Self> {
        let config: serde_json::Value = serde_json::from_str(config_json)
            .map_err(|e| CarbemError::Config(format!("Invalid JSON config: {}", e)))?;

        let provider = self.registry.create_provider(provider_name, config)?;
        self.providers.push(provider);
        Ok(self)
    }

    /// Build the final client (only available when configured)
    pub fn build(self) -> CarbemClient {
        CarbemClient {
            providers: self.providers,
        }
    }
}

/// Main client with type-safe guarantee of having providers
pub struct CarbemClient {
    providers: Vec<Box<dyn CarbonProvider + Send + Sync>>,
}

impl CarbemClient {
    /// Create a new builder
    pub fn builder() -> CarbemClientBuilder<Empty> {
        CarbemClientBuilder::new()
    }

    /// Query emissions from all configured providers
    pub async fn query_emissions(&self, query: &EmissionQuery) -> Result<Vec<CarbonEmission>> {
        for provider in &self.providers {
            if provider.name() == query.provider {
                return provider.get_emissions(query).await;
            }
        }
        Err(CarbemError::UnsupportedProvider(query.provider.clone()))
    }

    /// Get all available providers
    pub fn available_providers(&self) -> Vec<&str> {
        self.providers.iter().map(|p| p.name()).collect()
    }

    /// Check if a specific provider is configured
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.iter().any(|p| p.name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_safe_builder() {
        // This won't compile without configuring at least one provider
        // let client = CarbemClient::builder().build(); // ❌ Compile error!

        // This will compile
        let config = AzureConfig {
            access_token: "test".to_string(),
        };

        let client = CarbemClient::builder().with_azure(config).unwrap().build(); // ✅ Compiles!

        assert!(client.has_provider("azure"));
    }

    #[test]
    fn test_multiple_providers() {
        let config1 = AzureConfig {
            access_token: "test1".to_string(),
        };
        let config2 = AzureConfig {
            access_token: "test2".to_string(),
        };

        let client = CarbemClient::builder()
            .with_azure(config1)
            .unwrap()
            .with_azure(config2)
            .unwrap()
            .build();

        assert_eq!(client.available_providers().len(), 2);
    }
}
