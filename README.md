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

```rust
use carbem::CarbemClient;

#[tokio::main]
async fn main() -> carbem::Result<()> {
    let client = CarbemClient::new();
    
    // Usage examples will be added as providers are implemented
    // let emissions = client.get_emissions("aws", region, timeframe).await?;
    
    Ok(())
}
```

## Supported Providers

*This section will be updated as cloud providers are implemented:*

## Roadmap

- [ ] Core library infrastructure

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)


## Acknowledgments

This project aims to support sustainability efforts in cloud computing by making carbon emission data more accessible to developers and organizations.