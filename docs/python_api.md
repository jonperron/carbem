# get_emissions_py Function Documentation

## Overview

The `get_emissions_py` function is the main Python interface to the Carbem Rust library for retrieving carbon emission data from cloud providers.

## Function Signature

```python
carbem.get_emissions_py(provider: str, config_json: str, query_json: str) -> str
```

## Description

This function provides a Python interface to query carbon emission data from supported cloud providers. It is implemented in Rust using PyO3 bindings for optimal performance and safety.

The function makes HTTP requests to cloud provider APIs to retrieve carbon emission data based on the specified configuration and query parameters.

## Parameters

### provider (str)

- **Required**: Yes
- **Type**: String  
- **Description**: Name of the cloud provider to query
- **Supported values**: `"azure"`
- **Example**: `"azure"`

### config_json (str)

- **Required**: Yes
- **Type**: JSON string
- **Description**: Provider-specific configuration containing authentication credentials
- **Format**: Provider-dependent JSON object as string

**Azure Configuration Example**:

```json
{
  "access_token": "your-azure-bearer-token"
}
```

### query_json (str)

- **Required**: Yes  
- **Type**: JSON string
- **Description**: Query parameters specifying what emission data to retrieve
- **Format**: JSON object containing date range, regions, and optional filters

**Query Example**:

```json
{
  "start_date": "2024-01-01T00:00:00Z",
  "end_date": "2024-01-31T23:59:59Z", 
  "regions": ["subscription-id-1", "subscription-id-2"],
  "services": ["compute", "storage"],
  "resources": null
}
```

**Azure-Specific Query Configuration**:

When querying Azure, you can include additional Azure-specific parameters to customize the report type and carbon scope. These fields are part of the `AzureQueryConfig` struct:

```json
{
  "start_date": "2024-01-01T00:00:00Z",
  "end_date": "2024-01-31T23:59:59Z", 
  "regions": ["subscription-id"],
  "report_type": "ItemDetailsReport",
  "carbon_scope_list": ["Scope1", "Scope3", "Location"],
  "category_type": "Location",
  "order_by": "emissions",
  "page_size": 100,
  "sort_direction": "Desc",
  "top_items": 10
}
```

#### Azure Query Configuration Fields

| Field | Type | Required | Description | Default |
|-------|------|----------|-------------|---------|
| `report_type` | string | No | Type of Azure Carbon Emissions report | `"MonthlySummaryReport"` |
| `carbon_scope_list` | array of strings | No | Carbon scopes to include in the report | `["Scope1", "Scope2", "Scope3"]` |
| `start_date` | string (ISO 8601) | Yes | Start date for the emissions query period | None |
| `end_date` | string (ISO 8601) | Yes | End date for the emissions query period | None |
| `regions` | array of strings | Yes | Azure subscription IDs to query emissions from | None |

#### Valid Report Types

- `"OverallSummaryReport"` - Overall emissions summary
- `"MonthlySummaryReport"` - Monthly breakdown of emissions (default)
- `"TopItemsSummaryReport"` - Top emitting items summary
- `"TopItemsMonthlySummaryReport"` - Monthly top items summary
- `"ItemDetailsReport"` - Detailed item-level emissions data

#### Valid Carbon Scopes

- `"Scope1"` - Direct GHG emissions
- `"Scope2"` - Indirect emissions from purchased energy
- `"Scope3"` - Indirect emissions in value chain

## Return Value

### Type

`str` - JSON string containing emission data

### Format

Returns a JSON string containing an array of emission records. Each record includes:

- Provider information
- Regional data
- Service categorization
- Emission quantities in kg CO2 equivalent
- Time period information
- Additional metadata

### Example Response

```json
[
  {
    "provider": "azure",
    "region": "eastus", 
    "service": "compute",
    "emissions_kg_co2eq": 123.45,
    "time_period": {
      "start": "2024-01-01T00:00:00Z",
      "end": "2024-01-31T23:59:59Z"
    },
    "metadata": {
      "energy_kwh": 500.0,
      "grid_carbon_intensity": 0.5,
      "renewable_percentage": 25.0,
      "provider_data": {}
    }
  }
]
```

## Exceptions

The function may raise exceptions for various error conditions:

- **Authentication errors**: Invalid or expired access tokens
- **Permission errors**: Insufficient permissions for requested resources
- **Network errors**: Connection timeouts or network failures  
- **Validation errors**: Invalid date formats or parameter values
- **Rate limiting**: API rate limits exceeded
- **Service errors**: Cloud provider API errors

## Usage Examples

### Basic Usage

```python
import carbem
import json

# Configure Azure access
config = json.dumps({
    "access_token": "your-bearer-token"
})

# Query last 30 days for a subscription
query = json.dumps({
    "start_date": "2024-01-01T00:00:00Z",
    "end_date": "2024-01-31T23:59:59Z",
    "regions": ["your-subscription-id"]
})

# Get emission data
result = carbem.get_emissions_py("azure", config, query)
emissions = json.loads(result)
```

### With Error Handling

```python
try:
    result = carbem.get_emissions_py("azure", config, query)
    emissions = json.loads(result)
    print(f"Retrieved {len(emissions)} emission records")
except Exception as e:
    print(f"Error retrieving emissions: {e}")
```

### Processing Results

```python
result = carbem.get_emissions_py("azure", config, query)
emissions = json.loads(result)

# Calculate total emissions
total = sum(entry.get('emissions_kg_co2eq', 0) for entry in emissions)
print(f"Total emissions: {total:.2f} kg CO2eq")

# Group by service
services = {}
for entry in emissions:
    service = entry.get('service', 'unknown')
    if service not in services:
        services[service] = 0
    services[service] += entry.get('emissions_kg_co2eq', 0)

for service, amount in services.items():
    print(f"{service}: {amount:.2f} kg CO2eq")
```

## Version Compatibility

- **Python**: Requires Python 3.7+
- **Carbem**: Compatible with carbem-python 0.1.0+
- **Dependencies**: No additional Python dependencies required
