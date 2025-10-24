use serde::{Deserialize, Serialize};

// IBM Carbon Calculator API query parameters
#[derive(Debug, Clone, Serialize)]
pub struct IbmCarbonQuery {
    pub enterprise_id: String,
    
    // Date filtering - examples: "gte:2023-01", "lte:2023-03"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub month: Option<Vec<String>>,
    
    // Location filtering - examples: "Dallas", "Frankfurt", "Washington DC"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<String>>,
    
    // Service filtering - examples: "Cloud Object Storage", "Kubernetes Service"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub services: Option<Vec<String>>,
    
    // Grouping - examples: "month", "location", "service"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<String>,
}

// IBM Carbon Calculator API response structure
// TODO: Add actual response structure based on real API response
#[derive(Debug, Clone, Deserialize)]
pub struct IbmCarbonResponse {
    // This will be defined based on the actual JSON/CSV response format
    pub data: serde_json::Value,
}