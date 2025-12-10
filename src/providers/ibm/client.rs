use async_trait::async_trait;

use crate::error::{CarbemError, Result};
use crate::models::{CarbonEmission, EmissionMetadata, EmissionQuery, TimePeriod};
use crate::providers::CarbonProvider;
use crate::providers::config::ProviderQueryConfig;

use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use reqwest::{
    Client,
    header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue},
};

use super::models::*;

// IBM Carbon Calculator API base URL
const IBM_CARBON_API_BASE_URL: &str = "https://api.carbon-calculator.cloud.ibm.com";
const IBM_API_VERSION: &str = "v1";

// IBM Cloud provider
#[derive(Debug, Clone)]
pub struct IbmProvider {
    config: IbmConfig,
    http_client: Client,
}

impl IbmProvider {
    // Create a new IBM provider instance with configuration
    pub fn new(config: IbmConfig) -> Result<Self> {
        let http_client = Client::new();
        Ok(Self {
            config,
            http_client,
        })
    }

    // Convert EmissionQuery to IBM Carbon API request
    fn convert_emission_query_to_ibm_request(
        &self,
        query: &EmissionQuery,
    ) -> Result<IbmCarbonEmissionRequest> {
        // Extract IBM-specific configuration from provider_config (required)
        let ibm_config = match &query.provider_config {
            Some(ProviderQueryConfig::Ibm(config)) => config,
            Some(_) => {
                return Err(CarbemError::Config(
                    "provider_config must be IBM configuration for IBM provider".to_string(),
                ));
            }
            None => {
                return Err(CarbemError::Config(
                    "provider_config with IBM configuration is required".to_string(),
                ));
            }
        };

        // Validate the configuration
        ibm_config.validate().map_err(CarbemError::Config)?;

        // Convert time_period to month filters (format: "gte:2023-01", "lte:2023-03")
        let month_filters = self.build_month_filters(&query.time_period);

        // Build the request
        Ok(IbmCarbonEmissionRequest {
            enterprise_id: ibm_config.enterprise_id.clone(),
            month: if month_filters.is_empty() {
                None
            } else {
                Some(month_filters)
            },
            locations: if query.regions.is_empty() {
                None
            } else {
                Some(query.regions.clone())
            },
            services: query.services.clone(),
            enterprise_account_id: ibm_config.enterprise_account_id.clone(),
            group_by: ibm_config.group_by.as_ref().map(|g| g.as_str().to_string()),
            limit: ibm_config.limit,
            offset: ibm_config.offset,
        })
    }

    // Build month filters from time period
    fn build_month_filters(&self, time_period: &TimePeriod) -> Vec<String> {
        let mut filters = Vec::new();

        // Start month filter (gte:YYYY-MM)
        let start_month = time_period.start.format("%Y-%m").to_string();
        filters.push(format!("gte:{}", start_month));

        // End month filter (lte:YYYY-MM)
        let end_month = time_period.end.format("%Y-%m").to_string();
        filters.push(format!("lte:{}", end_month));

        filters
    }

    // Build authorization headers for IBM API requests
    fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        // Add authorization header (Bearer token from API key)
        let auth_value = format!("Bearer {}", self.config.api_key);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value)
                .map_err(|e| CarbemError::Config(format!("Invalid API key: {}", e)))?,
        );

        // Add Accept header for JSON response
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        Ok(headers)
    }

    // Build the API endpoint URL with query parameters
    fn build_endpoint_url(&self, request: &IbmCarbonEmissionRequest) -> String {
        let base_url = format!(
            "{}/{}/carbon_emissions",
            IBM_CARBON_API_BASE_URL, IBM_API_VERSION
        );

        let mut query_params = Vec::new();

        // Add enterprise_id as query parameter
        query_params.push(format!(
            "enterprise_id={}",
            urlencoding::encode(&request.enterprise_id)
        ));

        // Add month filters
        if let Some(months) = &request.month {
            for month in months {
                query_params.push(format!("month={}", urlencoding::encode(month)));
            }
        }

        // Add location filters (comma-separated in a single parameter)
        if let Some(locations) = &request.locations {
            let locations_str = locations.join(", ");
            query_params.push(format!("locations={}", urlencoding::encode(&locations_str)));
        }

        // Add service filters (comma-separated in a single parameter)
        if let Some(services) = &request.services {
            let services_str = services.join(", ");
            query_params.push(format!("services={}", urlencoding::encode(&services_str)));
        }

        // Add enterprise_account_id filter (optional, only for enterprise accounts)
        if let Some(enterprise_account_id) = &request.enterprise_account_id {
            query_params.push(format!(
                "enterprise_account_id={}",
                urlencoding::encode(enterprise_account_id)
            ));
        }

        // Add group_by
        if let Some(group_by) = &request.group_by {
            query_params.push(format!("group_by={}", group_by));
        }

        // Add pagination parameters
        if let Some(limit) = request.limit {
            query_params.push(format!("limit={}", limit));
        }

        if let Some(offset) = request.offset {
            query_params.push(format!("offset={}", offset));
        }

        format!("{}?{}", base_url, query_params.join("&"))
    }

    // Convert IBM emission data to carbem CarbonEmission
    fn convert_to_carbon_emission(
        &self,
        data: &IbmEmissionData,
        query_time_period: &TimePeriod,
    ) -> CarbonEmission {
        // Parse the month to create time period
        let emission_time_period = self
            .parse_month_to_time_period(&data.month.value)
            .unwrap_or_else(|| query_time_period.clone());

        // Create provider-specific data
        let mut provider_data = serde_json::Map::new();

        provider_data.insert(
            "account_id".to_string(),
            serde_json::Value::String(data.account_id.clone()),
        );

        // Add group_by info if present
        if let Some(group_by) = &data.group_by {
            provider_data.insert(
                "group_by_type".to_string(),
                serde_json::Value::String(group_by.group_type.clone()),
            );
            provider_data.insert(
                "group_by_value".to_string(),
                serde_json::Value::String(group_by.value.clone()),
            );
        }

        // Determine region from location or group_by value
        let region = data
            .location
            .clone()
            .or_else(|| {
                data.group_by.as_ref().and_then(|g| {
                    if g.group_type == "location" {
                        Some(g.value.clone())
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| "unknown".to_string());

        // Determine service from service field or group_by value
        let service = data.service.clone().or_else(|| {
            data.group_by.as_ref().and_then(|g| {
                if g.group_type == "service" {
                    Some(g.value.clone())
                } else {
                    None
                }
            })
        });

        // Convert energy consumption from Wh to kWh
        let energy_kwh = data.energy_consumption / 1000.0;

        CarbonEmission {
            provider: "ibm".to_string(),
            region,
            service,
            // API returns grams, convert to kg
            emissions_kg_co2eq: data.carbon_emission / 1000.0,
            time_period: emission_time_period,
            metadata: Some(EmissionMetadata {
                energy_kwh: Some(energy_kwh),
                grid_carbon_intensity: None,
                renewable_percentage: None,
                provider_data: Some(serde_json::Value::Object(provider_data)),
            }),
        }
    }

    // Parse month string (YYYY-MM) to TimePeriod
    fn parse_month_to_time_period(&self, month: &str) -> Option<TimePeriod> {
        // Parse "YYYY-MM" format
        let date_str = format!("{}-01", month);
        let start_date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok()?;

        // Calculate end of month
        let end_date = if start_date.month() == 12 {
            NaiveDate::from_ymd_opt(start_date.year() + 1, 1, 1)?.pred_opt()?
        } else {
            NaiveDate::from_ymd_opt(start_date.year(), start_date.month() + 1, 1)?.pred_opt()?
        };

        Some(TimePeriod {
            start: Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0)?),
            end: Utc.from_utc_datetime(&end_date.and_hms_opt(23, 59, 59)?),
        })
    }
}

#[async_trait]
impl CarbonProvider for IbmProvider {
    fn name(&self) -> &'static str {
        "ibm"
    }

    async fn get_emissions(&self, query: &EmissionQuery) -> Result<Vec<CarbonEmission>> {
        // Convert query to IBM format
        let ibm_request = self.convert_emission_query_to_ibm_request(query)?;

        // Build URL and headers
        let url = self.build_endpoint_url(&ibm_request);
        let headers = self.build_headers()?;

        // Make API request
        let response = self
            .http_client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| CarbemError::Api(format!("IBM API request failed: {}", e)))?;

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CarbemError::Api(format!(
                "IBM API returned error {}: {}",
                status, error_body
            )));
        }

        // Parse response
        let ibm_response: IbmCarbonEmissionResponse = response
            .json()
            .await
            .map_err(|e| CarbemError::Api(format!("Failed to parse IBM API response: {}", e)))?;

        // Convert to CarbonEmission
        let emissions: Vec<CarbonEmission> = ibm_response
            .carbon_emissions
            .iter()
            .map(|data| self.convert_to_carbon_emission(data, &query.time_period))
            .collect();

        Ok(emissions)
    }

    fn is_configured(&self) -> bool {
        !self.config.api_key.is_empty()
    }

    fn clone_provider(&self) -> Box<dyn CarbonProvider + Send + Sync> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_test_config() -> IbmConfig {
        IbmConfig {
            api_key: "test-api-key".to_string(),
        }
    }

    fn create_test_emission_query() -> EmissionQuery {
        EmissionQuery {
            provider: "ibm".to_string(),
            regions: vec!["Dallas".to_string(), "Frankfurt".to_string()],
            time_period: TimePeriod {
                start: Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
                end: Utc.with_ymd_and_hms(2023, 3, 31, 23, 59, 59).unwrap(),
            },
            services: Some(vec![
                "Cloud Object Storage".to_string(),
                "Kubernetes Service".to_string(),
            ]),
            resources: None,
            provider_config: Some(ProviderQueryConfig::Ibm(IbmQueryConfig {
                enterprise_id: "x2x261x8x5x84xxxx49x4891xx077xx9".to_string(),
                group_by: Some(IbmGroupBy::Month),
                enterprise_account_id: None,
                limit: Some(10),
                offset: None,
            })),
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
        };
        let provider = IbmProvider::new(empty_config).unwrap();
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_convert_emission_query_to_ibm_request() {
        let config = create_test_config();
        let provider = IbmProvider::new(config).unwrap();
        let query = create_test_emission_query();

        let ibm_request = provider
            .convert_emission_query_to_ibm_request(&query)
            .unwrap();

        assert_eq!(
            ibm_request.enterprise_id,
            "x2x261x8x5x84xxxx49x4891xx077xx9"
        );
        assert_eq!(
            ibm_request.locations,
            Some(vec!["Dallas".to_string(), "Frankfurt".to_string()])
        );
        assert_eq!(
            ibm_request.services,
            Some(vec![
                "Cloud Object Storage".to_string(),
                "Kubernetes Service".to_string()
            ])
        );
        assert_eq!(ibm_request.group_by, Some("month".to_string()));
        assert_eq!(ibm_request.limit, Some(10));

        // Check month filters
        let months = ibm_request.month.unwrap();
        assert!(months.contains(&"gte:2023-01".to_string()));
        assert!(months.contains(&"lte:2023-03".to_string()));
    }

    #[test]
    fn test_missing_provider_config() {
        let config = create_test_config();
        let provider = IbmProvider::new(config).unwrap();
        let mut query = create_test_emission_query();
        query.provider_config = None;

        let result = provider.convert_emission_query_to_ibm_request(&query);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("provider_config with IBM configuration is required")
        );
    }

    #[test]
    fn test_missing_enterprise_id() {
        let config = create_test_config();
        let provider = IbmProvider::new(config).unwrap();
        let mut query = create_test_emission_query();
        query.provider_config = Some(ProviderQueryConfig::Ibm(IbmQueryConfig {
            enterprise_id: "".to_string(),
            group_by: None,
            enterprise_account_id: None,
            limit: None,
            offset: None,
        }));

        let result = provider.convert_emission_query_to_ibm_request(&query);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("enterprise_id is required")
        );
    }

    #[test]
    fn test_build_endpoint_url() {
        let config = create_test_config();
        let provider = IbmProvider::new(config).unwrap();
        let query = create_test_emission_query();

        let ibm_request = provider
            .convert_emission_query_to_ibm_request(&query)
            .unwrap();
        let url = provider.build_endpoint_url(&ibm_request);

        // Check URL structure - enterprise_id is now a query param
        assert!(
            url.starts_with("https://api.carbon-calculator.cloud.ibm.com/v1/carbon_emissions?")
        );
        assert!(url.contains("enterprise_id=x2x261x8x5x84xxxx49x4891xx077xx9"));
        assert!(url.contains("month=gte%3A2023-01"));
        assert!(url.contains("month=lte%3A2023-03"));
        // Locations are comma-separated
        assert!(url.contains("locations=Dallas%2C%20Frankfurt"));
        // Services are comma-separated
        assert!(url.contains("services=Cloud%20Object%20Storage%2C%20Kubernetes%20Service"));
        assert!(url.contains("group_by=month"));
        assert!(url.contains("limit=10"));
    }

    #[test]
    fn test_build_month_filters() {
        let config = create_test_config();
        let provider = IbmProvider::new(config).unwrap();

        let time_period = TimePeriod {
            start: Utc.with_ymd_and_hms(2023, 1, 15, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2023, 3, 20, 23, 59, 59).unwrap(),
        };

        let filters = provider.build_month_filters(&time_period);
        assert_eq!(filters.len(), 2);
        assert_eq!(filters[0], "gte:2023-01");
        assert_eq!(filters[1], "lte:2023-03");
    }

    #[test]
    fn test_parse_month_to_time_period() {
        let config = create_test_config();
        let provider = IbmProvider::new(config).unwrap();

        let time_period = provider.parse_month_to_time_period("2024-02").unwrap();
        assert_eq!(
            time_period.start.format("%Y-%m-%d").to_string(),
            "2024-02-01"
        );
        assert_eq!(time_period.end.format("%Y-%m-%d").to_string(), "2024-02-29");
        // 2024 is a leap year
    }

    #[test]
    fn test_convert_to_carbon_emission() {
        let config = create_test_config();
        let provider = IbmProvider::new(config).unwrap();

        let data = IbmEmissionData {
            account_id: "x2x261x8x5x84xxxx49x4891xx077xx9".to_string(),
            carbon_emission: 2000.0,    // grams
            energy_consumption: 5000.0, // Wh
            month: IbmMonthInfo {
                value: "2023-01".to_string(),
                min: Some("2023-01".to_string()),
                max: Some("2023-02".to_string()),
            },
            group_by: Some(IbmGroupByInfo {
                group_type: "month".to_string(),
                value: "2023-01".to_string(),
            }),
            location: None,
            service: None,
        };

        let time_period = TimePeriod {
            start: Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2023, 3, 31, 23, 59, 59).unwrap(),
        };

        let emission = provider.convert_to_carbon_emission(&data, &time_period);

        assert_eq!(emission.provider, "ibm");
        assert_eq!(emission.region, "unknown"); // No location in this case
        assert_eq!(emission.service, None);
        // 2000 grams = 2 kg
        assert_eq!(emission.emissions_kg_co2eq, 2.0);
        // 5000 Wh = 5 kWh
        assert_eq!(emission.metadata.as_ref().unwrap().energy_kwh, Some(5.0));

        // Check provider_data
        let provider_data = emission
            .metadata
            .as_ref()
            .unwrap()
            .provider_data
            .as_ref()
            .unwrap();
        assert_eq!(
            provider_data.get("account_id").unwrap(),
            "x2x261x8x5x84xxxx49x4891xx077xx9"
        );
        assert_eq!(provider_data.get("group_by_type").unwrap(), "month");
        assert_eq!(provider_data.get("group_by_value").unwrap(), "2023-01");
    }

    #[test]
    fn test_convert_to_carbon_emission_with_location() {
        let config = create_test_config();
        let provider = IbmProvider::new(config).unwrap();

        let data = IbmEmissionData {
            account_id: "test-account".to_string(),
            carbon_emission: 1500.0,
            energy_consumption: 3000.0,
            month: IbmMonthInfo {
                value: "2023-02".to_string(),
                min: None,
                max: None,
            },
            group_by: Some(IbmGroupByInfo {
                group_type: "location".to_string(),
                value: "Dallas".to_string(),
            }),
            location: Some("Dallas".to_string()),
            service: Some("Cloud Object Storage".to_string()),
        };

        let time_period = TimePeriod {
            start: Utc.with_ymd_and_hms(2023, 2, 1, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2023, 2, 28, 23, 59, 59).unwrap(),
        };

        let emission = provider.convert_to_carbon_emission(&data, &time_period);

        assert_eq!(emission.provider, "ibm");
        assert_eq!(emission.region, "Dallas");
        assert_eq!(emission.service, Some("Cloud Object Storage".to_string()));
        assert_eq!(emission.emissions_kg_co2eq, 1.5); // 1500g = 1.5kg
    }
}
