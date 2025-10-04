use carbem::get_emissions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let access_token = env::var("AZURE_TOKEN").expect("Set AZURE_TOKEN environment variable");

    // This is how it would be called from Python/TypeScript
    let config_json = format!(
        r#"{{
        "access_token": "{}"
    }}"#,
        access_token
    );

    let payload_json = r#"{
        "start_date": "2024-09-01T00:00:00Z",
        "end_date": "2024-09-30T00:00:00Z",
        "regions": ["your-subscription-id"]
    }"#;

    let emissions = get_emissions("azure", &config_json, payload_json).await?;

    println!("Found {} emission records:", emissions.len());
    for emission in emissions {
        println!(
            "{} | {} | {:.4} kg CO2eq | {}",
            emission.region,
            emission.service.unwrap_or_else(|| "Unknown".to_string()),
            emission.emissions_kg_co2eq,
            emission.time_period.start.format("%Y-%m-%d")
        );
    }

    Ok(())
}
