use serde::{Deserialize, Serialize};

// Sort direction for ItemDetailsReport
#[derive(Debug, Clone)]
pub enum SortDirection {
    Asc,
    Desc,
}

// Date range for the carbon emission report
#[derive(Debug, Clone, Serialize)]
pub struct AzureDateRange {
    pub start: String, // Format: "YYYY-MM-DD"
    pub end: String,   // Format: "YYYY-MM-DD"
}

// Report types supported by Azure Carbon API
#[derive(Debug, Clone)]
pub enum AzureReportType {
    OverallSummary,
    MonthlySummary,
    TopItemsSummary,
    TopItemsMonthlySummary,
    ItemDetails,
}

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

/// Azure API response for carbon emission reports
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureCarbonEmissionReportResponse {
    pub(super) subscription_access_decision_list: Vec<AzureSubscriptionAccessDecision>,
    pub(super) value: Vec<AzureEmissionData>,
}
