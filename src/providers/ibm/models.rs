use serde::{Deserialize, Serialize};

// ============================================================================
// Generic structs
// ============================================================================

// IBM grouping options for carbon emissions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IbmGroupBy {
    #[default]
    Month,
    Location,
    Service,
    Account,
}

impl IbmGroupBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            IbmGroupBy::Month => "month",
            IbmGroupBy::Location => "location",
            IbmGroupBy::Service => "service",
            IbmGroupBy::Account => "account",
        }
    }
}

// ============================================================================
// Provider Configuration Types
// ============================================================================

// Configuration for IBM Cloud provider (authentication)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IbmConfig {
    pub api_key: String,
}

// ============================================================================
// Query Configuration Types
// ============================================================================

// IBM query configuration for Carbon Calculator API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IbmQueryConfig {
    // Mandatory enterprise ID for IBM Cloud
    pub enterprise_id: String,

    // Grouping option for results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<IbmGroupBy>,

    // Enterprise account ID to filter (optional)
    // Only applicable for enterprise accounts, not standard accounts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_account_id: Option<String>,

    // Pagination limit (optional, default is 10)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,

    // Pagination offset (optional, default is 0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
}

impl Default for IbmQueryConfig {
    fn default() -> Self {
        Self {
            enterprise_id: String::new(),
            group_by: Some(IbmGroupBy::Month),
            enterprise_account_id: None,
            limit: None,
            offset: None,
        }
    }
}

impl IbmQueryConfig {
    // Validates that all required fields are present
    pub fn validate(&self) -> Result<(), String> {
        // Validate mandatory enterprise_id
        if self.enterprise_id.is_empty() {
            return Err("enterprise_id is required and cannot be empty".to_string());
        }

        Ok(())
    }
}

// ============================================================================
// API Request Types
// ============================================================================

// IBM Carbon Calculator API query parameters
#[derive(Debug, Clone, Serialize)]
pub struct IbmCarbonEmissionRequest {
    pub(super) enterprise_id: String,

    // Date filtering - examples: "gte:2023-01", "lte:2023-03"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) month: Option<Vec<String>>,

    // Location filtering - examples: "Dallas", "Frankfurt", "Washington DC"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) locations: Option<Vec<String>>,

    // Service filtering - examples: "Cloud Object Storage", "Kubernetes Service"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) services: Option<Vec<String>>,

    // Enterprise account ID filter (optional, only for enterprise accounts)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) enterprise_account_id: Option<String>,

    // Grouping - examples: "month", "location", "service", "account"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) group_by: Option<String>,

    // Pagination limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) limit: Option<i32>,

    // Pagination offset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) offset: Option<i32>,
}

// ============================================================================
// API Response Types
// ============================================================================

// Month information in IBM response
#[allow(dead_code)] // min/max reserved for time range validation in future
#[derive(Debug, Clone, Deserialize)]
pub struct IbmMonthInfo {
    pub(super) value: String,
    #[serde(default)]
    pub(super) min: Option<String>,
    #[serde(default)]
    pub(super) max: Option<String>,
}

// Group by information in IBM response
#[derive(Debug, Clone, Deserialize)]
pub struct IbmGroupByInfo {
    #[serde(rename = "type")]
    pub(super) group_type: String,
    pub(super) value: String,
}

// IBM Carbon emission data point
#[derive(Debug, Clone, Deserialize)]
pub struct IbmEmissionData {
    // Account ID
    pub(super) account_id: String,

    // Carbon emissions in grams CO2e (note: API returns grams, we convert to kg)
    pub(super) carbon_emission: f64,

    // Energy consumption in Wh
    pub(super) energy_consumption: f64,

    // Month information
    pub(super) month: IbmMonthInfo,

    // Group by information
    #[serde(default)]
    pub(super) group_by: Option<IbmGroupByInfo>,

    // Location (when grouped by location)
    #[serde(default)]
    pub(super) location: Option<String>,

    // Service (when grouped by service)
    #[serde(default)]
    pub(super) service: Option<String>,
}

// Pagination link
#[allow(dead_code)] // href reserved for pagination navigation in future
#[derive(Debug, Clone, Deserialize)]
pub struct IbmPaginationLink {
    pub(super) href: String,
}

// IBM Carbon Calculator API response structure
#[allow(dead_code)] // Pagination fields reserved for future use
#[derive(Debug, Clone, Deserialize)]
pub struct IbmCarbonEmissionResponse {
    pub(super) carbon_emissions: Vec<IbmEmissionData>,

    // Total emissions across all results
    #[serde(default)]
    pub(super) total_emission: Option<f64>,

    // Pagination info
    #[serde(default)]
    pub(super) offset: Option<i32>,

    #[serde(default)]
    pub(super) limit: Option<i32>,

    #[serde(default)]
    pub(super) total_count: Option<i64>,

    // Pagination links
    #[serde(default)]
    pub(super) first: Option<IbmPaginationLink>,

    #[serde(default)]
    pub(super) last: Option<IbmPaginationLink>,

    #[serde(default)]
    pub(super) previous: Option<IbmPaginationLink>,

    #[serde(default)]
    pub(super) next: Option<IbmPaginationLink>,
}
