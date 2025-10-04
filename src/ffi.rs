//! Foreign Function Interface (FFI) layer for Python/TypeScript bindings
//!
//! This module provides simple JSON-based functions that can be easily
//! called from Python using PyO3 or from TypeScript using NAPI-RS.

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use serde_json;

use crate::client::CarbemClient;
use crate::error::{CarbemError, Result};
use crate::models::{CarbonEmission, EmissionQuery, TimePeriod};
use crate::providers::azure::AzureConfig;

/// FFI-friendly function to get emissions using JSON configuration and payload
///
/// This function is designed to be called from Python/TypeScript with simple string parameters:
/// - `provider`: Provider name (e.g., "azure")  
/// - `json_config`: JSON string with provider configuration
/// - `json_payload`: JSON string with query parameters
///
/// # Example
/// ```rust,no_run
/// use carbem::get_emissions;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = r#"{"access_token": "token"}"#;
///     let payload = r#"{"start_date": "2024-01-01T00:00:00Z", "end_date": "2024-02-01T00:00:00Z", "regions": ["sub-id"], "services": null, "resources": null}"#;
///     let emissions = get_emissions("azure", config, payload).await?;
///     Ok(())
/// }
/// ```
pub async fn get_emissions(
    provider: &str,
    json_config: &str,
    json_payload: &str,
) -> Result<Vec<CarbonEmission>> {
    let client = create_client_from_json(provider, json_config)?;
    let query = parse_emission_query_from_json(provider, json_payload)?;
    client.query_emissions(&query).await
}

/// Create a configured client from JSON configuration
fn create_client_from_json(provider: &str, json_config: &str) -> Result<CarbemClient> {
    let client = CarbemClient::new();

    match provider {
        "azure" => {
            let config: AzureConfig =
                serde_json::from_str(json_config).map_err(|e| CarbemError::Json(e))?;
            client.with_azure(config)
        }
        _ => Err(CarbemError::UnsupportedProvider(provider.to_string())),
    }
}

/// Parse EmissionQuery from JSON payload
fn parse_emission_query_from_json(provider: &str, json_payload: &str) -> Result<EmissionQuery> {
    let payload: HashMap<String, serde_json::Value> =
        serde_json::from_str(json_payload).map_err(|e| CarbemError::Json(e))?;

    let start_date = match payload.get("start_date") {
        Some(value) => match value.as_str() {
            Some(date_str) => DateTime::parse_from_rfc3339(date_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|_| {
                    CarbemError::Config(format!(
                        "Invalid start_date format '{}': expected RFC3339 format (e.g., '2024-01-01T00:00:00Z')",
                        date_str
                    ))
                })?,
            None => {
                return Err(CarbemError::Config(
                    "start_date must be a string".to_string(),
                ))
            }
        },
        None => Utc::now() - Duration::days(30), // Default to 30 days ago when absent
    };

    let end_date = match payload.get("end_date") {
        Some(value) => match value.as_str() {
            Some(date_str) => DateTime::parse_from_rfc3339(date_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|_| {
                    CarbemError::Config(format!(
                        "Invalid end_date format '{}': expected RFC3339 format (e.g., '2024-01-01T00:00:00Z')",
                        date_str
                    ))
                })?,
            None => {
                return Err(CarbemError::Config(
                    "end_date must be a string".to_string(),
                ))
            }
        },
        None => Utc::now(), // Default to now when absent
    };

    let regions = payload
        .get("regions")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let services = payload
        .get("services")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

    let resources = payload
        .get("resources")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

    Ok(EmissionQuery {
        provider: provider.to_string(),
        time_period: TimePeriod {
            start: start_date,
            end: end_date,
        },
        regions,
        services,
        resources,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_emission_query_from_json() {
        let json = r#"{
            "start_date": "2024-01-01T00:00:00Z",
            "end_date": "2024-02-01T00:00:00Z",
            "regions": ["sub-1", "sub-2"],
            "services": ["compute", "storage"]
        }"#;

        let query = parse_emission_query_from_json("azure", json).unwrap();

        assert_eq!(query.provider, "azure");
        assert_eq!(query.regions, vec!["sub-1", "sub-2"]);
        assert_eq!(
            query.services,
            Some(vec!["compute".to_string(), "storage".to_string()])
        );
        assert_eq!(query.resources, None);
    }

    #[test]
    fn test_parse_emission_query_invalid_dates() {
        // Test invalid start_date
        let json_invalid_start = r#"{
            "start_date": "invalid-date",
            "end_date": "2024-02-01T00:00:00Z",
            "regions": ["sub-1"]
        }"#;

        let result = parse_emission_query_from_json("azure", json_invalid_start);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid start_date format"));

        // Test invalid end_date
        let json_invalid_end = r#"{
            "start_date": "2024-01-01T00:00:00Z",
            "end_date": "not-a-date",
            "regions": ["sub-1"]
        }"#;

        let result = parse_emission_query_from_json("azure", json_invalid_end);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid end_date format"));

        // Test missing dates (should be OK - uses defaults)
        let json_no_dates = r#"{
            "regions": ["sub-1"]
        }"#;

        let result = parse_emission_query_from_json("azure", json_no_dates);
        assert!(result.is_ok());
        // When dates are missing, defaults are used (current time range)
        let query = result.unwrap();
        assert_eq!(query.regions, vec!["sub-1"]);
    }

    #[tokio::test]
    #[ignore] // Requires real Azure token
    async fn test_get_emissions_integration() {
        if let Ok(token) = std::env::var("AZURE_TOKEN") {
            let config = format!(r#"{{"access_token": "{}"}}"#, token);
            let payload = r#"{
                "start_date": "2024-01-01T00:00:00Z",
                "end_date": "2024-02-01T00:00:00Z",
                "regions": ["your-subscription-id"]
            }"#;

            let result = get_emissions("azure", &config, payload).await;
            assert!(result.is_ok());
        }
    }
}
