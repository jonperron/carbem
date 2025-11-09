use serde::{Deserialize, Serialize};

// ============================================================================
// Generic structs
// ============================================================================

// Date range for the carbon emission report
#[derive(Debug, Clone, Serialize)]
pub struct AzureDateRange {
    pub start: String, // Format: "YYYY-MM-DD"
    pub end: String,   // Format: "YYYY-MM-DD"
}

// Azure report types
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub enum AzureReportType {
    OverallSummaryReport,
    #[default]
    MonthlySummaryReport,
    TopItemsSummaryReport,
    TopItemsMonthlySummaryReport,
    ItemDetailsReport,
}

impl AzureReportType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AzureReportType::OverallSummaryReport => "OverallSummaryReport",
            AzureReportType::MonthlySummaryReport => "MonthlySummaryReport",
            AzureReportType::TopItemsSummaryReport => "TopItemsSummaryReport",
            AzureReportType::TopItemsMonthlySummaryReport => "TopItemsMonthlySummaryReport",
            AzureReportType::ItemDetailsReport => "ItemDetailsReport",
        }
    }
}

// Azure carbon scopes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum AzureCarbonScope {
    Scope1,
    Scope2,
    Scope3,
    Location,
    Service,
}

impl AzureCarbonScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            AzureCarbonScope::Scope1 => "Scope1",
            AzureCarbonScope::Scope2 => "Scope2",
            AzureCarbonScope::Scope3 => "Scope3",
            AzureCarbonScope::Location => "Location",
            AzureCarbonScope::Service => "Service",
        }
    }
}

// Azure sort direction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum AzureSortDirection {
    Asc,
    Desc,
}

impl AzureSortDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            AzureSortDirection::Asc => "Asc",
            AzureSortDirection::Desc => "Desc",
        }
    }
}

// ============================================================================
// Provider Configuration Types
// ============================================================================
// Configuration for Azure provider
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AzureConfig {
    pub access_token: String,
}

// ============================================================================
// Query Configuration Types
// ============================================================================

// Azure query configuration for Carbon Emissions API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureQueryConfig {
    // Report type for Azure Carbon Emissions API
    pub report_type: AzureReportType,

    // Carbon scope list for emissions calculation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carbon_scope_list: Option<Vec<AzureCarbonScope>>,

    // Category type for certain report types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_type: Option<String>,

    // Order by field for ItemDetailsReport
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<String>,

    // Page size for ItemDetailsReport
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,

    // Sort direction for ItemDetailsReport
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_direction: Option<AzureSortDirection>,

    // Top items count for summary reports
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_items: Option<i32>,

    // Mandatory subscription list - different from location_list
    // Format: List of subscription IDs (e.g., ["sub-id-1", "sub-id-2"])
    pub subscription_list: Vec<String>,

    // Optional filters - applicable to all report types

    // List of resource group URLs (format: /subscriptions/{subscriptionId}/resourcegroups/{resourceGroup}, lowercase)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_group_url_list: Option<Vec<String>>,

    // List of resource types (format: microsoft.{service}/{resourceType}, lowercase, e.g., "microsoft.storage/storageaccounts")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type_list: Option<Vec<String>>,

    // Pagination token for ItemDetailsReport (returned in previous response if more pages available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_token: Option<String>,
}

impl Default for AzureQueryConfig {
    fn default() -> Self {
        Self {
            report_type: AzureReportType::default(),
            subscription_list: vec![],
            carbon_scope_list: Some(vec![
                AzureCarbonScope::Scope1,
                AzureCarbonScope::Scope2,
                AzureCarbonScope::Scope3,
            ]),
            category_type: None,
            order_by: None,
            page_size: None,
            sort_direction: None,
            top_items: None,
            resource_group_url_list: None,
            resource_type_list: None,
            skip_token: None,
        }
    }
}

impl AzureQueryConfig {
    // Validates that all required fields for the specified report type are present
    pub fn validate(&self) -> Result<(), String> {
        // Validate mandatory subscription_list
        if self.subscription_list.is_empty() {
            return Err("subscription_list is required and cannot be empty".to_string());
        }

        // report_type is now mandatory (not Option), so it's always present
        match &self.report_type {
            AzureReportType::ItemDetailsReport => {
                // ItemDetailsReport requires: category_type, order_by, page_size, sort_direction
                if self.category_type.is_none() {
                    return Err("category_type is required for ItemDetailsReport".to_string());
                }
                if self.order_by.is_none() {
                    return Err("order_by is required for ItemDetailsReport".to_string());
                }
                if self.page_size.is_none() {
                    return Err("page_size is required for ItemDetailsReport".to_string());
                }
                // page_size is guaranteed to be Some at this point
                let page_size = self.page_size.unwrap();
                if !(1..=5000).contains(&page_size) {
                    return Err(
                        "page_size must be between 1 and 5000 for ItemDetailsReport".to_string()
                    );
                }
                if self.sort_direction.is_none() {
                    return Err("sort_direction is required for ItemDetailsReport".to_string());
                }
            }
            AzureReportType::TopItemsSummaryReport => {
                // TopItemsSummaryReport requires: category_type, top_items
                if self.category_type.is_none() {
                    return Err("category_type is required for TopItemsSummaryReport".to_string());
                }
                if self.top_items.is_none() {
                    return Err("top_items is required for TopItemsSummaryReport".to_string());
                }
                // top_items is guaranteed to be Some at this point
                let top_items = self.top_items.unwrap();
                if !(1..=10).contains(&top_items) {
                    return Err(
                        "top_items must be between 1 and 10 for TopItemsSummaryReport".to_string(),
                    );
                }
            }
            AzureReportType::TopItemsMonthlySummaryReport => {
                // TopItemsMonthlySummaryReport requires: category_type, top_items
                if self.category_type.is_none() {
                    return Err(
                        "category_type is required for TopItemsMonthlySummaryReport".to_string()
                    );
                }
                if self.top_items.is_none() {
                    return Err(
                        "top_items is required for TopItemsMonthlySummaryReport".to_string()
                    );
                }
                // top_items is guaranteed to be Some at this point
                let top_items = self.top_items.unwrap();
                if !(1..=10).contains(&top_items) {
                    return Err(
                        "top_items must be between 1 and 10 for TopItemsMonthlySummaryReport"
                            .to_string(),
                    );
                }
            }
            _ => {
                // MonthlySummaryReport, OverallSummaryReport and any future report types
                // have no additional required fields beyond the common ones
            }
        }

        Ok(())
    }
}

// ============================================================================
// Report request and response types
// ============================================================================

// Azure Carbon Emission Reports request payload
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureCarbonEmissionReportRequest {
    // Mandatory fields for all report types
    pub(super) carbon_scope_list: Vec<String>,
    pub(super) date_range: AzureDateRange,
    pub(super) report_type: String,
    pub(super) subscription_list: Vec<String>,
    // Mandatory field for ItemDetailsQueryFilter, TopItemsMonthlySummaryReportQueryFilter, TopItemsSummaryReportQueryFilter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) category_type: Option<String>,
    // Mandatory fields for ItemDetailsQueryFilter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) order_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) page_size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) sort_direction: Option<String>,
    // Mandatory fields for TopItemsMonthlySummaryReportQueryFilter, TopItemsSummaryReportQueryFilter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) top_items: Option<i32>,
    // Optional fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) location_list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) resource_group_url_list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) resource_type_list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) skip_token: Option<String>,
}

// Subscription access decision in Azure response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureSubscriptionAccessDecision {
    pub(super) subscription_id: String,
    pub(super) decision: String,
    #[serde(default)]
    pub(super) denial_reason: Option<String>,
}

// Emission data from Azure API
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureEmissionData {
    pub(super) data_type: String,
    pub(super) latest_month_emissions: f64,
    pub(super) previous_month_emissions: f64,
    pub(super) month_over_month_emissions_change_ratio: f64,
    pub(super) monthly_emissions_change_value: f64,
    #[serde(default)]
    pub(super) date: Option<String>, // Format: "YYYY-MM-DD", for MonthlySummaryReport
    #[serde(default)]
    pub(super) carbon_intensity: Option<f64>, // For MonthlySummaryReport
    #[serde(default)]
    pub(super) item_name: Option<String>, // For TopItemsSummaryReport, TopItemsMonthlySummaryReport & ItemDetailsReport(e.g., "east us", "west us")
    #[serde(default)]
    pub(super) category_type: Option<String>, // For TopItemsSummaryReport, TopItemsMonthlySummaryReport & ItemDetailsReport (e.g., "Location")
}

// Azure API response for carbon emission reports
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureCarbonEmissionReportResponse {
    #[serde(default)]
    pub(super) subscription_access_decision_list: Option<Vec<AzureSubscriptionAccessDecision>>,
    pub(super) value: Vec<AzureEmissionData>,
}
