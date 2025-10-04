//! Provider Registry Pattern for dynamic provider management

use std::collections::HashMap;

use crate::error::{CarbemError, Result};
use crate::providers::azure::{AzureConfig, AzureProvider};
use crate::providers::CarbonProvider;

/// Type alias for provider factory functions
type ProviderFactory =
    Box<dyn Fn(serde_json::Value) -> Result<Box<dyn CarbonProvider + Send + Sync>> + Send + Sync>;

/// Registry for carbon emission providers
pub struct ProviderRegistry {
    factories: HashMap<String, ProviderFactory>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new() -> Self {
        let mut registry = Self {
            factories: HashMap::new(),
        };

        // Register built-in providers
        registry.register_azure();

        registry
    }

    /// Register Azure provider factory
    fn register_azure(&mut self) {
        let factory: ProviderFactory = Box::new(|config_json| {
            let config: AzureConfig = serde_json::from_value(config_json)
                .map_err(|e| CarbemError::Config(format!("Invalid Azure config: {}", e)))?;

            let provider = AzureProvider::new(config)?;
            Ok(Box::new(provider) as Box<dyn CarbonProvider + Send + Sync>)
        });

        self.factories.insert("azure".to_string(), factory);
    }

    /// Register a custom provider factory
    pub fn register_provider<F>(&mut self, name: &str, factory: F)
    where
        F: Fn(serde_json::Value) -> Result<Box<dyn CarbonProvider + Send + Sync>>
            + Send
            + Sync
            + 'static,
    {
        self.factories.insert(name.to_string(), Box::new(factory));
    }

    /// Create a provider instance
    pub fn create_provider(
        &self,
        name: &str,
        config: serde_json::Value,
    ) -> Result<Box<dyn CarbonProvider + Send + Sync>> {
        let factory = self
            .factories
            .get(name)
            .ok_or_else(|| CarbemError::UnsupportedProvider(name.to_string()))?;

        factory(config)
    }

    /// Get list of available providers
    pub fn available_providers(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_registry_creation() {
        let registry = ProviderRegistry::new();
        let providers = registry.available_providers();
        assert!(providers.contains(&"azure".to_string()));
    }

    #[test]
    fn test_azure_provider_creation() {
        let registry = ProviderRegistry::new();
        let config = json!({"access_token": "test-token"});

        let result = registry.create_provider("azure", config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unknown_provider() {
        let registry = ProviderRegistry::new();
        let config = json!({});

        let result = registry.create_provider("unknown", config);
        assert!(result.is_err());
    }
}
