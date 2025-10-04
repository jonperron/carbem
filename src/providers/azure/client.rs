use async_trait::async_trait;
use chrono::{DateTime, Datelike, TimeZone, Utc};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde::Deserialize;

use crate::error::{CarbemError, Result};
use crate::models::{CarbonEmission, EmissionMetadata, EmissionQuery, TimePeriod};
use crate::providers::CarbonProvider;

use super::models::*;

// Azure Management API base URL
const AZURE_MANAGEMENT_BASE_URL: &str = "https://management.azure.com";
const CARBON_API_VERSION: &str = "2025-04-01";

// Configuration for Azure provider
#[derive(Debug, Clone, Deserialize)]
pub struct AzureConfig {
    pub access_token: String,
}

// Azure Carbon Optimization provider
#[derive(Debug, Clone)]
pub struct AzureProvider {
    config: AzureConfig,
    http_client: Client,
}

impl AzureProvider {
    // Create a new Azure provider instance with configuration
    pub fn new(config: AzureConfig) -> Result<Self> {
        let http_client = Client::new();
        Ok(Self {
            config,
            http_client,
        })
    }

    // Convert EmissionQuery to Azure-specific request format
    fn convert_emission_query_to_azure_request(
        &self,
        query: &EmissionQuery,
    ) -> Result<AzureCarbonEmissionReportRequest> {
        // Convert time period to Azure date format (YYYY-MM-DD)
        let start_date = query.time_period.start.format("%Y-%m-%d").to_string();
        let end_date = query.time_period.end.format("%Y-%m-%d").to_string();

        let date_range = AzureDateRange {
            start: start_date,
            end: end_date,
        };

        // Default to MonthlySummaryReport for now - can be made configurable later
        let report_type = "MonthlySummaryReport".to_string();

        // Use regions from query as subscription IDs
        let subscription_list = query.regions.clone();

        // Default carbon scopes - can be made configurable later
        let carbon_scope_list = vec!["Scope1".to_string(), "Scope3".to_string()];

        Ok(AzureCarbonEmissionReportRequest {
            report_type,
            subscription_list,
            carbon_scope_list,
            date_range,
            // Optional fields - set to None for now
            category_type: None,
            top_items: None,
            order_by: None,
            sort_direction: None,
            page_size: None,
            location_list: None,
            resource_group_url_list: None,
            resource_type_list: None,
            skip_token: None,
        })
    }

    // Build authorization headers for Azure API requests
    fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        // Add authorization header
        let auth_value = format!("Bearer {}", self.config.access_token);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value)
                .map_err(|e| CarbemError::Config(format!("Invalid access token: {}", e)))?,
        );

        // Add content type
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        Ok(headers)
    }

    fn build_request_payload(
        &self,
        query: &AzureCarbonEmissionReportRequest,
    ) -> AzureCarbonEmissionReportRequest {
        // At the moment, does nothing as we are still using only one struct
        return query.clone();
    }

    ///Convert Azure emission data to carbem CarbonEmission
    fn convert_to_carbon_emission(
        &self,
        data: &AzureEmissionData,
        subscription_id: &str,
        date_range: &AzureDateRange,
    ) -> CarbonEmission {
        // Create metadata with Azure-specific information
        let mut provider_data = serde_json::Map::new();
        provider_data.insert(
            "dataType".to_string(),
            serde_json::Value::String(data.data_type.clone()),
        );
        provider_data.insert(
            "previousMonthEmissions".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(data.previous_month_emissions).unwrap(),
            ),
        );
        provider_data.insert(
            "monthOverMonthEmissionsChangeRatio".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(data.month_over_month_emissions_change_ratio).unwrap(),
            ),
        );
        provider_data.insert(
            "monthlyEmissionsChangeValue".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(data.monthly_emissions_change_value).unwrap(),
            ),
        );

        // Add date if available
        if let Some(date) = &data.date {
            provider_data.insert("date".to_string(), serde_json::Value::String(date.clone()));
        }

        // Add item name and category type if available (for TopItemsSummaryReport)
        if let Some(item_name) = &data.item_name {
            provider_data.insert(
                "itemName".to_string(),
                serde_json::Value::String(item_name.clone()),
            );
        }
        if let Some(category_type) = &data.category_type {
            provider_data.insert(
                "categoryType".to_string(),
                serde_json::Value::String(category_type.clone()),
            );
        }

        // Create specific time period for this data point if date is provided
        let specific_time_period = if let Some(date) = &data.date {
            // Parse the date and create a month-specific time period
            match DateTime::parse_from_str(
                &format!("{}T00:00:00+00:00", date),
                "%Y-%m-%dT%H:%M:%S%z",
            ) {
                Ok(start_date) => {
                    let start = start_date.with_timezone(&Utc);
                    // Create end of month
                    let end = if start.month() == 12 {
                        start
                            .with_year(start.year() + 1)
                            .unwrap()
                            .with_month(1)
                            .unwrap()
                    } else {
                        start.with_month(start.month() + 1).unwrap()
                    };
                    TimePeriod { start, end }
                }
                Err(_) => {
                    // Fallback: convert DateRange to TimePeriod using start and end
                    let start = DateTime::parse_from_str(
                        &format!("{}T00:00:00+00:00", date_range.start),
                        "%Y-%m-%dT%H:%M:%S%z",
                    )
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| {
                        // Final fallback to a fixed date for testing consistency
                        DateTime::parse_from_str("2024-01-01T00:00:00+00:00", "%Y-%m-%dT%H:%M:%S%z")
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap())
                    });
                    let end = DateTime::parse_from_str(
                        &format!("{}T00:00:00+00:00", date_range.end),
                        "%Y-%m-%dT%H:%M:%S%z",
                    )
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| {
                        // Final fallback to a fixed date for testing consistency
                        DateTime::parse_from_str("2024-01-02T00:00:00+00:00", "%Y-%m-%dT%H:%M:%S%z")
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap())
                    });
                    TimePeriod { start, end }
                }
            }
        } else {
            // Convert DateRange to TimePeriod using start and end
            let start = DateTime::parse_from_str(
                &format!("{}T00:00:00+00:00", date_range.start),
                "%Y-%m-%dT%H:%M:%S%z",
            )
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| {
                // Final fallback to a fixed date for testing consistency
                DateTime::parse_from_str("2024-01-01T00:00:00+00:00", "%Y-%m-%dT%H:%M:%S%z")
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap())
            });
            let end = DateTime::parse_from_str(
                &format!("{}T00:00:00+00:00", date_range.end),
                "%Y-%m-%dT%H:%M:%S%z",
            )
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| {
                // Final fallback to a fixed date for testing consistency
                DateTime::parse_from_str("2024-01-02T00:00:00+00:00", "%Y-%m-%dT%H:%M:%S%z")
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap())
            });
            TimePeriod { start, end }
        };

        let metadata = EmissionMetadata {
            energy_kwh: None,                             // Not provided by Azure API
            grid_carbon_intensity: data.carbon_intensity, // Use Azure's carbon intensity
            renewable_percentage: None,                   // Not provided by Azure API
            provider_data: Some(serde_json::Value::Object(provider_data)),
        };

        // Use item_name as region if available (for location-based reports), otherwise use subscription_id
        let region = data
            .item_name
            .as_ref()
            .unwrap_or(&subscription_id.to_string())
            .clone();

        // Use category_type as service if available, otherwise default to "overall"
        let service = data
            .category_type
            .as_ref()
            .map(|ct| ct.to_lowercase())
            .or_else(|| Some("overall".to_string()));

        CarbonEmission {
            provider: "azure".to_string(),
            region,
            service,
            emissions_kg_co2eq: data.latest_month_emissions,
            time_period: specific_time_period,
            metadata: Some(metadata),
        }
    }

    async fn request_carbon_emissions(
        &self,
        query: &AzureCarbonEmissionReportRequest,
    ) -> Result<Vec<CarbonEmission>> {
        let url = format!(
            "{}/providers/Microsoft.Carbon/carbonEmissionReports?api-version={}",
            AZURE_MANAGEMENT_BASE_URL, CARBON_API_VERSION
        );

        let headers = self.build_headers()?;
        let payload = self.build_request_payload(query);

        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .map_err(|e| CarbemError::Http(e))?;

        // Check if request was successful
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CarbemError::Provider(format!(
                "Azure API request failed with status {}: {}",
                status, body
            )));
        }

        let azure_response: AzureCarbonEmissionReportResponse =
            response.json().await.map_err(CarbemError::Http)?;

        // Check for access decisions and collect denied subscriptions info
        let mut allowed_subscriptions = Vec::new();
        let mut denied_subscriptions = Vec::new();
        for decision in &azure_response.subscription_access_decision_list {
            if decision.decision == "Allowed" {
                allowed_subscriptions.push(decision.subscription_id.clone());
                continue;
            } else {
                let reason = decision
                    .denial_reason
                    .as_ref()
                    .map(|r| format!(" ({})", r))
                    .unwrap_or_default();
                denied_subscriptions.push(format!("{}{}", decision.subscription_id, reason));
            }
        }

        // Log denied subscriptions but don't fail the entire request if some subscriptions are allowed
        if !denied_subscriptions.is_empty() {
            // If ALL subscriptions are denied, return an error
            if denied_subscriptions.len() == azure_response.subscription_access_decision_list.len()
            {
                return Err(CarbemError::Auth(format!(
                    "Access denied for all subscriptions: {}",
                    denied_subscriptions.join(", ")
                )));
            }
            // Otherwise, we can continue with the allowed subscriptions
        }

        // Convert Azure response to carbem format
        let mut emissions = Vec::new();
        for data in azure_response.value {
            for subscription_id in &allowed_subscriptions {
                let emission =
                    self.convert_to_carbon_emission(&data, &subscription_id, &query.date_range);
                emissions.push(emission);
            }
        }

        // Sort emissions by date if available (newest first)
        emissions.sort_by(|a, b| b.time_period.start.cmp(&a.time_period.start));

        Ok(emissions)
    }
}

#[async_trait]
impl CarbonProvider for AzureProvider {
    fn name(&self) -> &'static str {
        "azure"
    }

    async fn get_emissions(&self, query: &EmissionQuery) -> Result<Vec<CarbonEmission>> {
        if query.provider != "azure" {
            return Err(CarbemError::Config(
                "Query provider must be 'azure' for AzureProvider".to_string(),
            ));
        }

        if query.regions.is_empty() {
            return Err(CarbemError::Config(
                "At least one subscriptionId must be specified in the query".to_string(),
            ));
        }

        // Convert EmissionQuery to Azure request format
        let azure_request = self.convert_emission_query_to_azure_request(query)?;

        self.request_carbon_emissions(&azure_request).await
    }

    fn is_configured(&self) -> bool {
        !self.config.access_token.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EmissionQuery, TimePeriod};
    use chrono::{TimeZone, Utc};

    fn create_test_provider() -> AzureProvider {
        let config = AzureConfig {
            access_token: "test-token".to_string(),
        };
        AzureProvider::new(config).unwrap()
    }

    fn create_test_emission_query() -> EmissionQuery {
        EmissionQuery {
            provider: "azure".to_string(),
            regions: vec!["00000000-0000-0000-0000-000000000000".to_string()],
            time_period: TimePeriod {
                start: Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2024, 5, 1, 0, 0, 0).unwrap(),
            },
            services: None,
            resources: None,
        }
    }

    #[test]
    fn test_azure_provider_creation() {
        let config = AzureConfig {
            access_token: "test-token".to_string(),
        };
        let provider = AzureProvider::new(config).unwrap();

        assert_eq!(provider.name(), "azure");
        assert!(provider.is_configured());
    }

    #[test]
    fn test_azure_provider_not_configured_with_empty_token() {
        let config = AzureConfig {
            access_token: "".to_string(),
        };
        let provider = AzureProvider::new(config).unwrap();

        assert!(!provider.is_configured());
    }

    #[test]
    fn test_convert_emission_query_to_azure_request() {
        let provider = create_test_provider();
        let query = create_test_emission_query();

        let azure_request = provider
            .convert_emission_query_to_azure_request(&query)
            .unwrap();

        assert_eq!(azure_request.report_type, "MonthlySummaryReport");
        assert_eq!(
            azure_request.subscription_list,
            vec!["00000000-0000-0000-0000-000000000000"]
        );
        assert_eq!(azure_request.carbon_scope_list, vec!["Scope1", "Scope3"]);
        assert_eq!(azure_request.date_range.start, "2024-03-01");
        assert_eq!(azure_request.date_range.end, "2024-05-01");
        assert_eq!(azure_request.category_type, None);
        assert_eq!(azure_request.top_items, None);
        assert_eq!(azure_request.order_by, None);
        assert_eq!(azure_request.sort_direction, None);
        assert_eq!(azure_request.page_size, None);
    }

    #[test]
    fn test_build_headers() {
        let provider = create_test_provider();
        let headers = provider.build_headers().unwrap();

        assert!(headers.contains_key("authorization"));
        assert!(headers.contains_key("content-type"));

        let auth_header = headers.get("authorization").unwrap().to_str().unwrap();
        assert_eq!(auth_header, "Bearer test-token");

        let content_type = headers.get("content-type").unwrap().to_str().unwrap();
        assert_eq!(content_type, "application/json");
    }

    #[test]
    fn test_build_headers_invalid_token() {
        let config = AzureConfig {
            access_token: "invalid\ntoken".to_string(),
        };
        let provider = AzureProvider::new(config).unwrap();

        let result = provider.build_headers();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid access token"));
    }

    #[test]
    fn test_build_request_payload() {
        let provider = create_test_provider();
        let query = create_test_emission_query();

        let azure_request = provider
            .convert_emission_query_to_azure_request(&query)
            .unwrap();
        let payload = provider.build_request_payload(&azure_request);

        // Since build_request_payload currently just clones the input, they should be identical
        assert_eq!(payload.report_type, azure_request.report_type);
        assert_eq!(payload.subscription_list, azure_request.subscription_list);
        assert_eq!(payload.carbon_scope_list, azure_request.carbon_scope_list);
        assert_eq!(payload.date_range.start, azure_request.date_range.start);
        assert_eq!(payload.date_range.end, azure_request.date_range.end);
    }

    #[test]
    fn test_convert_to_carbon_emission_basic() {
        let provider = create_test_provider();
        let date_range = AzureDateRange {
            start: "2024-03-01".to_string(),
            end: "2024-05-01".to_string(),
        };

        let azure_data = AzureEmissionData {
            data_type: "MonthlySummaryData".to_string(),
            latest_month_emissions: 0.1,
            previous_month_emissions: 0.05,
            month_over_month_emissions_change_ratio: 1.0,
            monthly_emissions_change_value: 0.05,
            date: Some("2024-05-01".to_string()),
            carbon_intensity: Some(22.0),
            item_name: None,
            category_type: None,
        };

        let emission =
            provider.convert_to_carbon_emission(&azure_data, "test-subscription", &date_range);

        assert_eq!(emission.provider, "azure");
        assert_eq!(emission.region, "test-subscription");
        assert_eq!(emission.service, Some("overall".to_string()));
        assert_eq!(emission.emissions_kg_co2eq, 0.1);
        assert!(emission.metadata.is_some());

        let metadata = emission.metadata.unwrap();
        assert_eq!(metadata.grid_carbon_intensity, Some(22.0));
        assert!(metadata.provider_data.is_some());

        let provider_data = metadata.provider_data.unwrap();
        assert_eq!(provider_data["dataType"], "MonthlySummaryData");
        assert_eq!(provider_data["date"], "2024-05-01");
        assert_eq!(provider_data["previousMonthEmissions"], 0.05);
        assert_eq!(provider_data["monthOverMonthEmissionsChangeRatio"], 1.0);
        assert_eq!(provider_data["monthlyEmissionsChangeValue"], 0.05);
    }

    #[test]
    fn test_convert_to_carbon_emission_with_item_name() {
        let provider = create_test_provider();
        let date_range = AzureDateRange {
            start: "2024-03-01".to_string(),
            end: "2024-05-01".to_string(),
        };

        let azure_data = AzureEmissionData {
            data_type: "TopItemsSummaryData".to_string(),
            latest_month_emissions: 0.1,
            previous_month_emissions: 0.05,
            month_over_month_emissions_change_ratio: 1.0,
            monthly_emissions_change_value: 0.05,
            date: None,
            carbon_intensity: None,
            item_name: Some("east us".to_string()),
            category_type: Some("Location".to_string()),
        };

        let emission =
            provider.convert_to_carbon_emission(&azure_data, "test-subscription", &date_range);

        assert_eq!(emission.provider, "azure");
        assert_eq!(emission.region, "east us"); // Should use item_name as region
        assert_eq!(emission.service, Some("location".to_string())); // Should use category_type (lowercased)
        assert_eq!(emission.emissions_kg_co2eq, 0.1);

        let metadata = emission.metadata.unwrap();
        let provider_data = metadata.provider_data.unwrap();
        assert_eq!(provider_data["itemName"], "east us");
        assert_eq!(provider_data["categoryType"], "Location");
    }

    #[test]
    fn test_convert_to_carbon_emission_with_date_parsing() {
        let provider = create_test_provider();
        let date_range = AzureDateRange {
            start: "2024-03-01".to_string(),
            end: "2024-05-01".to_string(),
        };

        let azure_data = AzureEmissionData {
            data_type: "MonthlySummaryData".to_string(),
            latest_month_emissions: 0.1,
            previous_month_emissions: 0.05,
            month_over_month_emissions_change_ratio: 1.0,
            monthly_emissions_change_value: 0.05,
            date: Some("2024-05-01".to_string()),
            carbon_intensity: Some(22.0),
            item_name: None,
            category_type: None,
        };

        let emission =
            provider.convert_to_carbon_emission(&azure_data, "test-subscription", &date_range);

        // Check that the time period was created from the date (May 1 to June 1)
        let expected_start = Utc.with_ymd_and_hms(2024, 5, 1, 0, 0, 0).unwrap();
        let expected_end = Utc.with_ymd_and_hms(2024, 6, 1, 0, 0, 0).unwrap();

        assert_eq!(emission.time_period.start, expected_start);
        assert_eq!(emission.time_period.end, expected_end);
    }

    #[test]
    fn test_get_emissions_wrong_provider() {
        let provider = create_test_provider();
        let mut query = create_test_emission_query();
        query.provider = "aws".to_string();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(provider.get_emissions(&query));

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Query provider must be 'azure'"));
    }

    #[test]
    fn test_get_emissions_no_regions() {
        let provider = create_test_provider();
        let mut query = create_test_emission_query();
        query.regions = vec![];

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(provider.get_emissions(&query));

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one subscriptionId must be specified"));
    }

    #[test]
    fn test_month_end_calculation_december() {
        let provider = create_test_provider();
        let date_range = AzureDateRange {
            start: "2024-12-01".to_string(),
            end: "2024-12-31".to_string(),
        };

        let azure_data = AzureEmissionData {
            data_type: "MonthlySummaryData".to_string(),
            latest_month_emissions: 0.1,
            previous_month_emissions: 0.05,
            month_over_month_emissions_change_ratio: 1.0,
            monthly_emissions_change_value: 0.05,
            date: Some("2024-12-01".to_string()),
            carbon_intensity: None,
            item_name: None,
            category_type: None,
        };

        let emission =
            provider.convert_to_carbon_emission(&azure_data, "test-subscription", &date_range);

        // December should roll over to January of the next year
        let expected_start = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let expected_end = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        assert_eq!(emission.time_period.start, expected_start);
        assert_eq!(emission.time_period.end, expected_end);
    }

    #[test]
    fn test_overall_summary_data_conversion() {
        let provider = create_test_provider();
        let date_range = AzureDateRange {
            start: "2024-03-01".to_string(),
            end: "2024-05-01".to_string(),
        };

        let azure_data = AzureEmissionData {
            data_type: "OverallSummaryData".to_string(),
            latest_month_emissions: 0.1,
            previous_month_emissions: 0.05,
            month_over_month_emissions_change_ratio: 1.0,
            monthly_emissions_change_value: 0.05,
            date: None,
            carbon_intensity: None,
            item_name: None,
            category_type: None,
        };

        let emission =
            provider.convert_to_carbon_emission(&azure_data, "test-subscription", &date_range);

        assert_eq!(emission.provider, "azure");
        assert_eq!(emission.region, "test-subscription"); // Should use subscription_id when no item_name
        assert_eq!(emission.service, Some("overall".to_string())); // Should default to "overall"
        assert_eq!(emission.emissions_kg_co2eq, 0.1);

        // Should use the original date range when no specific date is provided
        let expected_start = Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap();
        let expected_end = Utc.with_ymd_and_hms(2024, 5, 1, 0, 0, 0).unwrap();

        assert_eq!(emission.time_period.start, expected_start);
        assert_eq!(emission.time_period.end, expected_end);

        let metadata = emission.metadata.unwrap();
        let provider_data = metadata.provider_data.unwrap();
        assert_eq!(provider_data["dataType"], "OverallSummaryData");
    }

    #[tokio::test]
    #[ignore] // Ignore by default as this requires a real Azure token
    async fn test_get_emissions_integration() {
        // This test would require actual Azure credentials
        // Only run when AZURE_ACCESS_TOKEN environment variable is set
        if let Ok(access_token) = std::env::var("AZURE_ACCESS_TOKEN") {
            let config = AzureConfig { access_token };
            let provider = AzureProvider::new(config).unwrap();

            let query = EmissionQuery {
                provider: "azure".to_string(),
                regions: vec!["your-subscription-id".to_string()],
                time_period: TimePeriod {
                    start: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
                    end: Utc.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap(),
                },
                services: None,
                resources: None,
            };

            let result = provider.get_emissions(&query).await;
            assert!(result.is_ok());
        }
    }
}
