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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_type: Option<AzureReportType>,

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
}

impl Default for AzureQueryConfig {
    fn default() -> Self {
        Self {
            report_type: Some(AzureReportType::default()),
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
        }
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
