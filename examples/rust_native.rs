use carbem::models::{EmissionQuery, TimePeriod};
use carbem::CarbemClient;
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
        regions: vec!["your-subscription-id".to_string()], // Replace with your subscription ID
        time_period: TimePeriod {
            start: Utc.with_ymd_and_hms(2024, 9, 1, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2024, 9, 30, 23, 59, 59).unwrap(),
        },
        services: None,
        resources: None,
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
