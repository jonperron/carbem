# Carbem

A Rust library for retrieving carbon emission values from cloud providers.

## Overview

Carbem provides a unified interface for querying carbon emission data from various cloud service providers. This library helps developers build more environmentally conscious applications by making it easy to access and analyze the carbon footprint of cloud infrastructure.

## Features

- 🌍 **Multi-provider support**: Unified API for different cloud providers
- ⚡ **Async/await**: Built with modern async Rust for high performance
- 🔒 **Type-safe**: Leverages Rust's type system for reliable carbon data handling
- 🚀 **Easy to use**: Simple and intuitive API design

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
carbem = "0.1.0"
```

## Quick Start

### Python FFI Compatible API (Recommended)

The library provides a simple 3-parameter function designed for easy Python integration:

```rust
use carbem::get_emissions;

#[tokio::main]
async fn main() -> carbem::Result<()> {
    // JSON configuration for Azure
    let config_json = r#"{
        "subscription_ids": ["00000000-0000-0000-0000-000000000000"],
        "access_token": "your_azure_bearer_token_here",
        "tenant_id": "your_tenant_id_here",
        "report_type": "monthly"
    }"#;
    
    // JSON payload for the query
    let payload_json = r#"{
        "start_date": "2024-01-01T00:00:00Z",
        "end_date": "2024-02-01T00:00:00Z",
        "regions": ["eastus", "westus"],
        "services": null,
        "resources": null
    }"#;
    
    // Simple function call - perfect for Python FFI
    let emissions = get_emissions("azure", config_json, payload_json).await?;
    
    for emission in emissions {
        println!("Service: {}, Emissions: {} kg CO2eq", 
                 emission.service.unwrap_or_default(),
                 emission.emissions_kg_co2eq);
    }
    
    Ok(())
}
```

### Standalone Rust API

For standalone Rust applications, use environment variables:

```rust
use carbem::CarbemClient;

#[tokio::main]
async fn main() -> carbem::Result<()> {
    // Automatically loads from .env file and environment variables
    let client = CarbemClient::from_env()?;
    
    // Simple query with sensible defaults
    let emissions = client.query_emissions(
        None, // start_date: defaults to 30 days ago
        None, // end_date: defaults to now  
        None, // regions: use all available regions
        None, // services: no filter
        None, // resources: no filter
    ).await?;
    
    for emission in emissions {
        println!("Service: {}, Emissions: {} kg CO2eq", 
                 emission.service.unwrap_or_default(),
                 emission.emissions_kg_co2eq);
    }
    
    Ok(())
}
```

Create a `.env` file in your project root:

```env
# Azure Carbon Emissions Configuration
CARBEM_AZURE_ACCESS_TOKEN=your_azure_bearer_token_here
```

## Configuration Parameters

### Environment Variables (for Standalone Rust)

- `CARBEM_AZURE_ACCESS_TOKEN`: Azure access token

### Object-Oriented API (Advanced Usage)

```rust
use carbem::{CarbemClient, AzureConfig, EmissionQuery, TimePeriod};
use chrono::{Utc, Duration};

#[tokio::main]
async fn main() -> carbem::Result<()> {
    // Create a client and configure Azure provider
    let mut client = CarbemClient::new();
    
    let config = AzureConfig {
        access_token: "your-bearer-token".to_string(),
    };
    
    client.set_azure_provider(config)?;
    
    // Query carbon emissions for the last 30 days
    let query = EmissionQuery {
        provider: "azure".to_string(),
        regions: vec![], // Use all available regions
        time_period: TimePeriod {
            start: Utc::now() - Duration::days(30),
            end: Utc::now(),
        },
    };
    
    let emissions = client.get_emissions(query).await?;
    
    for emission in emissions {
        println!("Date: {}, Region: {}, Emissions: {} kg CO2eq", 
                 emission.time_period.start.format("%Y-%m-%d"),
                 emission.region, 
                 emission.emissions_kg_co2eq);
    }
    
    Ok(())
}
```

## Supported Providers

### Microsoft Azure ✅

- **Carbon Emission Reports API**: Integrated with Azure's Carbon Emission Reports API
- **Multiple Report Types**: Support for Overall Summary and Monthly Summary reports
- **Flexible Date Ranges**: Query emissions for custom time periods
- **Region Filtering**: Filter results by specific Azure regions
- **Comprehensive Testing**: Full test suite ensuring reliability

### Planned Providers

- [ ] Amazon Web Services (AWS)
- [ ] Google Cloud Platform (GCP)
- [ ] Additional providers planned

## Roadmap

- [x] Core library infrastructure
- [x] Azure Carbon Emission Reports API integration
- [x] Python FFI compatible API design
- [x] Environment-based configuration
- [ ] Additional Azure report types (TopItems, ItemDetails)
- [ ] AWS provider implementation
- [ ] Google Cloud Platform provider

## Testing

The library includes a comprehensive test suite:

```bash
# Run all tests
cargo test

# Run specific Azure provider tests
cargo test providers::azure

# Run with output
cargo test -- --nocapture
```

Test coverage includes:

- Provider creation and configuration
- Query conversion and validation
- Date parsing and time period handling
- Data conversion from Azure API responses  
- Error handling for invalid configurations

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

This project is licensed under Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)

## Acknowledgments

This project aims to support sustainability efforts in cloud computing by making carbon emission data more accessible to developers and organizations.

