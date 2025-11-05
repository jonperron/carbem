# Carbem

A Rust library for retrieving carbon emission values from cloud providers.

## Overview

Carbem provides a unified interface for querying carbon emission data from various cloud service providers. This library helps developers build more environmentally conscious applications by making it easy to access and analyze the carbon footprint of cloud infrastructure.

## Features

- 🌍 **Multi-provider support**: Unified API for different cloud providers
- ⚡ **Async/await**: Built with modern async Rust for high performance
- 🔒 **Type-safe**: Leverages Rust's type system for reliable carbon data handling
- 🚀 **Easy to use**: Simple and intuitive API design
- 🐍 **Python Bindings**: Native Python integration via PyO3
- 🔧 **Flexible Filtering**: Filter by regions, services, and resources

## Installation

### Rust Library

Add this to your `Cargo.toml`:

```toml
[dependencies]
carbem = "0.2.0"
```

### Python Package

Install from PyPI:

```bash
pip install carbem-python
```

For development setup with maturin:

```bash
pip install maturin
maturin develop
```

## Quick Start

### Using Rust

For standalone Rust applications, use the builder pattern with environment variables:

```rust
use carbem::{CarbemClient, EmissionQuery, TimePeriod};
use chrono::{Utc, Duration};

#[tokio::main]
async fn main() -> carbem::Result<()> {
    // Configure client from environment variables
    let client = CarbemClient::new()
        .with_azure_from_env()?;
    
    // Create a query
    let query = EmissionQuery {
        provider: "azure".to_string(),
        regions: vec!["subscription-id".to_string()],
        time_period: TimePeriod {
            start: Utc::now() - Duration::days(30),
            end: Utc::now(),
        },
        services: Some(vec!["compute".to_string(), "storage".to_string()]),
        resources: None,
    };
    
    let emissions = client.query_emissions(&query).await?;
    
    for emission in emissions {
        println!("Service: {}, Emissions: {} kg CO2eq", 
                 emission.service.unwrap_or_default(),
                 emission.emissions_kg_co2eq);
    }
    
    Ok(())
}
```

### Using Python

For Python applications, use the `get_emissions_py` function:

```python
import carbem
import json
from datetime import datetime, timedelta

# Azure configuration
config = json.dumps({
    "access_token": "your-azure-bearer-token"
})

# Query for last 30 days
end_date = datetime.utcnow()
start_date = end_date - timedelta(days=30)

query = json.dumps({
    "start_date": start_date.strftime("%Y-%m-%dT%H:%M:%SZ"),
    "end_date": end_date.strftime("%Y-%m-%dT%H:%M:%SZ"),
    "regions": ["your-subscription-id"],
})

# Get emissions data
result = carbem.get_emissions_py("azure", config, query)
emissions = json.loads(result)

print(f"Found {len(emissions)} emission records")
```

Create a `.env` file in your project root:

```env
# Azure Carbon Emissions Configuration
CARBEM_AZURE_ACCESS_TOKEN=your_azure_bearer_token_here
# OR alternatively use:
# AZURE_TOKEN=your_azure_bearer_token_here
```

## Configuration Parameters

### Environment Variables (for Standalone Rust)

- `CARBEM_AZURE_ACCESS_TOKEN`: Azure access token
- `AZURE_TOKEN`: Alternative Azure access token variable

### Python Configuration

For Python applications, configuration is passed as JSON strings to the `get_emissions_py` function. See the [Python API Documentation](docs/python_api.md) for detailed configuration examples and usage patterns.

### Azure Configuration (AzureConfig)

The Azure provider requires minimal configuration:

```rust
use carbem::AzureConfig;

let config = AzureConfig {
    access_token: "your-bearer-token".to_string(),
};
```

### Object-Oriented API (Advanced Usage)

```rust
use carbem::{CarbemClient, AzureConfig, EmissionQuery, TimePeriod};
use chrono::{Utc, Duration};

#[tokio::main]
async fn main() -> carbem::Result<()> {
    // Create a client and configure Azure provider
    let config = AzureConfig {
        access_token: "your-bearer-token".to_string(),
    };
    
    let client = CarbemClient::new()
        .with_azure(config)?;
    
    // Query carbon emissions for the last 30 days
    let query = EmissionQuery {
        provider: "azure".to_string(),
        regions: vec!["subscription-id".to_string()], // Use your subscription IDs
        time_period: TimePeriod {
            start: Utc::now() - Duration::days(30),
            end: Utc::now(),
        },
        services: None,
        resources: None,
    };
    
    let emissions = client.query_emissions(&query).await?;
    
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

- **Report Types**: All report type from [the API](https://learn.microsoft.com/en-us/azure/carbon-optimization/api-export-data?source=recommendations&tabs=OverallSummaryReport#report-types) are supported.
- **Queries**: All [query parameters](https://learn.microsoft.com/en-us/azure/carbon-optimization/api-export-data?source=recommendations&tabs=OverallSummaryReport#export-emissions-api-parameters) are supported.

###  Google Cloud Platform

Google Cloud Platform is not supported at the moment (October 11th 2025). Data are available only after exporting them to BigQuery as discussed in [this page](https://cloud.google.com/carbon-footprint/docs/api). Thus, one will need to make a query to the BigQuery API, which makes a standard implementation not possible at the moment.

### Amazon Web Services (AWS)

AWS is not supported at the moment (October 11th 2025). Data are available in S3 buckets as discussed in [this page](https://aws.amazon.com/fr/blogs/aws-cloud-financial-management/export-and-visualize-carbon-emissions-data-from-your-aws-accounts/). An endpoint existed but was discontinued on July 23rd 2025 ([ref](https://github.com/aws-samples/experimental-programmatic-access-ccft)).

## Roadmap

- [x] Core library infrastructure
- [x] Azure Carbon Emission Reports API integration
- [ ] Amazon Web Services (AWS)
- [ ] Google Cloud Platform (GCP)
- [ ] Additional providers planned

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

## Documentation

### Rust Documentation

- API Documentation: Available on [docs.rs](https://docs.rs/carbem/)

### Python Documentation

- [Python API Reference](docs/python_api.md) - Detailed function documentation and usage patterns  

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

This project is licensed under Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)

## Acknowledgments

This project aims to support sustainability efforts in cloud computing by making carbon emission data more accessible to developers and organizations.
