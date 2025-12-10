use carbem::CarbemClient;
use carbem::models::{EmissionQuery, TimePeriod};
use carbem::{AzureCarbonScope, AzureQueryConfig, AzureReportType, ProviderQueryConfig};
use chrono::{TimeZone, Utc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Option 1: Configure from environment
    let client = CarbemClient::builder().with_azure_from_env()?.build();

    // Option 2: Manual configuration
    // let access_token = env::var("AZURE_TOKEN")?;
    // let config = AzureConfig { access_token };
    // let client = CarbemClient::builder()
    //     .with_azure(config)?
    //     .build();

    let query = EmissionQuery {
        provider: "azure".to_string(),
        regions: vec!["eastus".to_string(), "westus".to_string()], // Location list (regions)
        time_period: TimePeriod {
            start: Utc.with_ymd_and_hms(2024, 9, 1, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2024, 9, 30, 23, 59, 59).unwrap(),
        },
        services: None,
        resources: None,
        // Type-safe configuration for Azure (required)
        provider_config: Some(ProviderQueryConfig::Azure(AzureQueryConfig {
            report_type: AzureReportType::MonthlySummaryReport,
            subscription_list: vec!["your-subscription-id".to_string()], // Replace with your subscription ID
            carbon_scope_list: Some(vec![AzureCarbonScope::Scope1, AzureCarbonScope::Scope3]),
            category_type: None,
            order_by: None,
            page_size: None,
            sort_direction: None,
            top_items: None,
            resource_group_url_list: None,
            resource_type_list: None,
            skip_token: None,
        })),
    };

    println!("Querying Azure carbon emissions...");

    match client.query_emissions(&query).await {
        Ok(emissions) => {
            println!("\n✅ Found {} emission records:", emissions.len());
            for emission in emissions {
                println!(
                    "  📍 {} | 🏷️  {} | 💨 {:.4} kg CO2eq | 📅 {}",
                    emission.region,
                    emission.service.unwrap_or_else(|| "overall".to_string()),
                    emission.emissions_kg_co2eq,
                    emission.time_period.start.format("%Y-%m-%d")
                );
            }
        }
        Err(e) => {
            eprintln!("❌ Error querying emissions: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
