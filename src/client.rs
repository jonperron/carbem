use crate::error::{CarbemError, Result};
use crate::models::{CarbonEmission, EmissionQuery, TimePeriod};
use crate::providers::azure::client::{AzureConfig, AzureProvider};
use crate::providers::azure::models::AzureReportType;
use crate::providers::CarbonProvider;
use std::env;

pub struct CarbemClient {
    azure_provider: Option<AzureProvider>,
}

pub async fn get_emissions(
    provider: &str,
    json_config: &str,
    json_payload: &str,
) -> Result<Vec<CarbonEmission>> {
    match provider.to_lowercase().as_str() {
        "azure" => get_azure_emissions_from_json(json_config, json_payload).await,
        _ => Err(CarbemError::UnsupportedProvider(provider.to_string())),
    }
}

pub async fn get_azure_emissions_from_json(
    json_config: &str,
    json_payload: &str,
) -> Result<Vec<CarbonEmission>> {
    #[derive(serde::Deserialize)]
    struct AzureConfigJson {
        subscription_ids: Vec<String>,
        access_token: String,
        tenant_id: Option<String>,
        report_type: Option<String>,
    }

    #[derive(serde::Deserialize)]
    struct QueryPayload {
        start_date: Option<String>,
        end_date: Option<String>,
        regions: Option<Vec<String>>,
        services: Option<Vec<String>>,
        resources: Option<Vec<String>>,
    }

    let config_json: AzureConfigJson = serde_json::from_str(json_config)
        .map_err(|e| CarbemError::ConfigError(format!("Invalid config JSON: {}", e)))?;

    let payload: QueryPayload = serde_json::from_str(json_payload)
        .map_err(|e| CarbemError::ConfigError(format!("Invalid payload JSON: {}", e)))?;

    let report_type = match config_json.report_type.as_deref().unwrap_or("overall") {
        "overall" => AzureReportType::OverallSummary,
        "monthly" => AzureReportType::MonthlySummary,
        _ => AzureReportType::OverallSummary,
    };

    let config = AzureConfig {
        access_token: config_json.access_token,
    };

    let provider = AzureProvider::new(config)?;

    let start_date = payload
        .start_date
        .as_ref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));

    let end_date = payload
        .end_date
        .as_ref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|| chrono::Utc::now());

    let query = EmissionQuery {
        provider: "azure".to_string(),
        regions: payload.regions.unwrap_or_default(),
        time_period: TimePeriod {
            start: start_date,
            end: end_date,
        },
    };

    provider.get_emissions(&query).await
}

impl CarbemClient {
    pub fn new() -> Self {
        Self {
            azure_provider: None,
        }
    }

    pub fn from_env() -> Result<Self> {
        let _ = dotenv::dotenv();
        let mut client = Self::new();
        client.load_azure_from_env()?;
        Ok(client)
    }

    pub fn load_azure_from_env(&mut self) -> Result<()> {
        let subscription_ids_str = env::var("CARBEM_AZURE_SUBSCRIPTION_IDS").map_err(|_| {
            CarbemError::ConfigError(
                "CARBEM_AZURE_SUBSCRIPTION_IDS environment variable not set".to_string(),
            )
        })?;

        let subscription_ids: Vec<String> = subscription_ids_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let access_token = env::var("CARBEM_AZURE_ACCESS_TOKEN").map_err(|_| {
            CarbemError::ConfigError(
                "CARBEM_AZURE_ACCESS_TOKEN environment variable not set".to_string(),
            )
        })?;

        let config = AzureConfig { access_token };

        self.set_azure_provider(config)
    }

    pub fn set_azure_provider(&mut self, config: AzureConfig) -> Result<()> {
        let provider = AzureProvider::new(config)?;
        self.azure_provider = Some(provider);
        Ok(())
    }

    pub async fn get_emissions(&self, query: EmissionQuery) -> Result<Vec<CarbonEmission>> {
        match query.provider.as_str() {
            "azure" => {
                if let Some(provider) = &self.azure_provider {
                    provider.get_emissions(&query).await
                } else {
                    Err(CarbemError::Config(
                        "No Azure provider configured".to_string(),
                    ))
                }
            }
            _ => Err(CarbemError::Provider(format!(
                "Unsupported provider: {}",
                query.provider
            ))),
        }
    }

    pub async fn query_emissions(
        &self,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
        regions: Option<Vec<String>>,
        services: Option<Vec<String>>,
        resources: Option<Vec<String>>,
    ) -> Result<Vec<CarbonEmission>> {
        let start = start_date.unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
        let end = end_date.unwrap_or_else(|| chrono::Utc::now());

        let query = EmissionQuery {
            provider: "azure".to_string(),
            regions: regions.unwrap_or_default(),
            time_period: TimePeriod { start, end },
        };

        self.get_emissions(query).await
    }
}

impl Default for CarbemClient {
    fn default() -> Self {
        Self::new()
    }
}
