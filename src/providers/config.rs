use crate::providers::azure::AzureQueryConfig;
use serde::{Deserialize, Serialize};

/// Provider-specific configuration enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", content = "config")]
pub enum ProviderQueryConfig {
    /// Azure-specific query configuration
    #[serde(rename = "azure")]
    Azure(AzureQueryConfig),
}
