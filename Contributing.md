# Contributing to Carbem

Thank you for considering contributing to Carbem! This project aims to make carbon emission data more accessible to developers and organizations working towards sustainability goals.

You will find in the following sections the ground rules and guidelines to help provide a quality tool to the open-source community.

## Ground Rules

By contributing to this project:

* **Be respectful** with others and remember that this project is maintained by volunteers in their free time.
* **Create an issue first** - If you want to make a significant change, please create an issue first so we can track and discuss your proposed changes.
* **Test your changes** - Make sure that any changes you provide are properly tested and don't break existing functionality.
* **Write clear commit messages** - Keep the history of your branch clean and descriptive.
* **Follow coding standards** - Ensure your code follows Rust best practices and project conventions.

## Getting Started

### Prerequisites

* Rust 1.70+ (edition 2021)
* An Azure account with Carbon Emission Reports API access (for testing Azure integration)

### Setting up the development environment

1. Clone the repository:

   ```bash
   git clone https://github.com/jonperron/carbem.git
   cd carbem
   ```

2. Install dependencies:

   ```bash
   cargo build
   ```

3. Set up environment variables for testing (optional):

   ```bash
   cp .env.example .env
   # Edit .env with your Azure credentials
   ```

## Testing

We maintain comprehensive test coverage to ensure reliability:

### Running all tests

```bash
cargo test
```

### Running specific test suites

```bash
# Test only Azure provider
cargo test providers::azure

# Test specific functionality
cargo test client::tests
```

### Integration tests

Some tests require actual Azure credentials and are marked as `#[ignore]` by default. To run them:

```bash
# Set your Azure token
export AZURE_TOKEN="your_token_here"

# Run ignored tests
cargo test -- --ignored
```

## Code Style and Standards

### Rust Guidelines

* Follow standard Rust formatting: `cargo fmt`
* Use meaningful variable and function names
* Add documentation for public APIs
* Include examples in documentation when helpful

### Commit Messages

Use conventional commit format:

```text
type(scope): brief description

More detailed explanation if needed.

Fixes #123
```

Examples:

* `feat(azure): add support for TopItemsSummary reports`
* `fix(client): handle empty subscription lists correctly`
* `docs(readme): update installation instructions`

## Adding New Providers

To add support for a new cloud provider:

1. Create a new module in `src/providers/`
2. Implement the `CarbonProvider` trait
3. Add comprehensive tests
4. Update the registry in `src/providers/registry.rs`
5. Update documentation and examples

See the Azure provider implementation as a reference.
