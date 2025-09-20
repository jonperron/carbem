use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Represents carbon emission data from a cloud provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonEmission {
    /// The cloud provider (e.g., "aws", "azure", "gcp")
    pub provider: String,
    
    /// The region where the emissions occurred
    pub region: String,
    
    /// The service or resource type
    pub service: Option<String>,
    
    /// Carbon emissions in kilograms of CO2 equivalent
    pub emissions_kg_co2eq: f64,
    
    /// The time period for which emissions are reported
    pub time_period: TimePeriod,
    
    /// Additional metadata
    pub metadata: Option<EmissionMetadata>,
}

/// Time period for carbon emission measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    /// Start of the measurement period
    pub start: DateTime<Utc>,
    
    /// End of the measurement period
    pub end: DateTime<Utc>,
}

/// Additional metadata for carbon emissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmissionMetadata {
    /// Energy consumption in kWh
    pub energy_kwh: Option<f64>,
    
    /// Grid carbon intensity (gCO2eq/kWh)
    pub grid_carbon_intensity: Option<f64>,
    
    /// Renewable energy percentage
    pub renewable_percentage: Option<f64>,
    
    /// Additional provider-specific data
    pub provider_data: Option<serde_json::Value>,
}

/// Configuration for querying carbon emissions
#[derive(Debug, Clone)]
pub struct EmissionQuery {
    /// The cloud provider to query
    pub provider: String,
    
    /// The region(s) to include
    pub regions: Vec<String>,
    
    /// The time period to query
    pub time_period: TimePeriod,
    
    /// Optional service filter
    pub services: Option<Vec<String>>,
    
    /// Optional resource filter
    pub resources: Option<Vec<String>>,
}