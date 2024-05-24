use serde::Deserialize;

#[derive(Deserialize)]
pub struct ReportConfig {
    pub region: String,
    pub report_query_role_arn: String,
    pub lambda_name: String,
    pub cost_allocation_tag: String,
}
