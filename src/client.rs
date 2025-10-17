//! Type-safe builder pattern for CarbemClient

use crate::error::{CarbemError, Result};
use crate::models::{CarbonEmission, EmissionQuery};
use crate::providers::azure::AzureConfig;
use crate::providers::ibm::IbmConfig;
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
    #[allow(clippy::new_without_default)]
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

    /// Add IBM Cloud provider with explicit config
    pub fn with_ibm(mut self, config: IbmConfig) -> Result<CarbemClientBuilder<Configured>> {
        let provider = self.registry.create_provider("ibm", json!(config))?;
        self.providers.push(provider);

        Ok(CarbemClientBuilder {
            registry: self.registry,
            providers: self.providers,
            _state: PhantomData,
        })
    }

    /// Add IBM Cloud provider from environment
    pub fn with_ibm_from_env(self) -> Result<CarbemClientBuilder<Configured>> {
        let api_key = std::env::var("IBM_API_KEY")
            .or_else(|_| std::env::var("CARBEM_IBM_API_KEY"))
            .map_err(|_| {
                CarbemError::Config(
                    "IBM_API_KEY or CARBEM_IBM_API_KEY environment variable not set"
                        .to_string(),
                )
            })?;

        let account_id = std::env::var("IBM_ACCOUNT_ID")
            .or_else(|_| std::env::var("CARBEM_IBM_ACCOUNT_ID"))
            .map_err(|_| {
                CarbemError::Config(
                    "IBM_ACCOUNT_ID or CARBEM_IBM_ACCOUNT_ID environment variable not set"
                        .to_string(),
                )
            })?;

        let config = IbmConfig {
            api_key,
            enterprise_id: account_id, // Using account_id from env as enterprise_id
        };
        self.with_ibm(config)
    }
}

impl CarbemClientBuilder<Configured> {
    /// Add another Azure provider (for multiple subscriptions)
    pub fn with_azure(mut self, config: AzureConfig) -> Result<Self> {
        let provider = self.registry.create_provider("azure", json!(config))?;
        self.providers.push(provider);
        Ok(self)
    }

    /// Add IBM Cloud provider (for additional accounts)
    pub fn with_ibm(mut self, config: IbmConfig) -> Result<Self> {
        let provider = self.registry.create_provider("ibm", json!(config))?;
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

impl Clone for CarbemClient {
    fn clone(&self) -> Self {
        Self {
            providers: self.providers.iter().map(|p| p.clone_provider()).collect(),
        }
    }
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
    fn test_ibm_provider_builder() {
        let config = IbmConfig {
            api_key: "test-api-key".to_string(),
            enterprise_id: "test-enterprise-id".to_string(),
        };

        let client = CarbemClient::builder().with_ibm(config).unwrap().build();

        assert!(client.has_provider("ibm"));
    }

    #[test]
    fn test_multiple_providers() {
        let azure_config = AzureConfig {
            access_token: "test-azure".to_string(),
        };
        let ibm_config = IbmConfig {
            api_key: "test-ibm-key".to_string(),
            enterprise_id: "test-ibm-enterprise".to_string(),
        };

        let client = CarbemClient::builder()
            .with_azure(azure_config)
            .unwrap()
            .with_ibm(ibm_config)
            .unwrap()
            .build();

        assert_eq!(client.available_providers().len(), 2);
        assert!(client.has_provider("azure"));
        assert!(client.has_provider("ibm"));
    }
}
