use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{CarbemError, Result};
use crate::models::{CarbonEmission, EmissionQuery};
use crate::providers::CarbonProvider;

use super::models::*;

// IBM Carbon Calculator API base URL
const IBM_CARBON_API_BASE_URL: &str = "https://api.carbon-calculator.cloud.ibm.com";
const IBM_API_VERSION: &str = "v1";

// Configuration for IBM Cloud provider
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IbmConfig {
    pub api_key: String,
    pub enterprise_id: String, // IBM uses enterprise_id instead of account_id
}

// IBM Cloud provider
#[derive(Debug, Clone)]
pub struct IbmProvider {
    config: IbmConfig,
}

impl IbmProvider {
    // Create a new IBM provider instance with configuration
    pub fn new(config: IbmConfig) -> Result<Self> {
        Ok(Self { config })
    }

    // Convert EmissionQuery to IBM query parameters
    fn build_query_params(&self, query: &EmissionQuery) -> IbmCarbonQuery {
        // TODO: Convert EmissionQuery to IBM Carbon API format
        // - time_period -> month filtering (gte/lte)
        // - regions -> locations
        // - services -> services
        
        IbmCarbonQuery {
            enterprise_id: self.config.enterprise_id.clone(),
            month: None, // TODO: Convert time_period to month filters
            locations: if query.regions.is_empty() { None } else { Some(query.regions.clone()) },
            services: query.services.clone(),
            group_by: Some("month".to_string()), // Default grouping
        }
    }

    // Build the API endpoint URL with query parameters
    fn build_endpoint_url(&self, query_params: &IbmCarbonQuery) -> Result<String> {
        let base_url = format!("{}/{}/carbon_emissions", IBM_CARBON_API_BASE_URL, IBM_API_VERSION);
        
        // TODO: Build query string from IbmCarbonQuery
        // Example: ?month=gte:2023-01&month=lte:2023-03&locations=Dallas,Frankfurt&services=Cloud%20Object%20Storage&group_by=month&enterprise_id=xxx
        
        let url_with_params = format!("{}?enterprise_id={}", base_url, query_params.enterprise_id);
        Ok(url_with_params)
    }
}

#[async_trait]
impl CarbonProvider for IbmProvider {
    fn name(&self) -> &'static str {
        "ibm"
    }

    async fn get_emissions(&self, _query: &EmissionQuery) -> Result<Vec<CarbonEmission>> {
        // TODO: Implement IBM Cloud carbon emissions API integration
        Err(CarbemError::Other("IBM provider not yet implemented".to_string()))
    }

    fn is_configured(&self) -> bool {
        !self.config.api_key.is_empty() && !self.config.enterprise_id.is_empty()
    }

    fn clone_provider(&self) -> Box<dyn CarbonProvider + Send + Sync> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> IbmConfig {
        IbmConfig {
            api_key: "test-api-key".to_string(),
            enterprise_id: "test-enterprise-id".to_string(),
        }
    }

    #[test]
    fn test_ibm_provider_creation() {
        let config = create_test_config();
        let provider = IbmProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_ibm_provider_name() {
        let config = create_test_config();
        let provider = IbmProvider::new(config).unwrap();
        assert_eq!(provider.name(), "ibm");
    }

    #[test]
    fn test_ibm_provider_is_configured() {
        let config = create_test_config();
        let provider = IbmProvider::new(config).unwrap();
        assert!(provider.is_configured());

        let empty_config = IbmConfig {
            api_key: "".to_string(),
            enterprise_id: "test-enterprise-id".to_string(),
        };
        let provider = IbmProvider::new(empty_config).unwrap();
        assert!(!provider.is_configured());
    }
}