use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct ReportConfig {
    pub region: String,
    pub report_query_role_arn: String,
    pub lambda_name: String,
    pub cost_allocation_tags: Vec<TagKeyValues>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AccountReportConfig {
    pub region: String,
    pub report_query_role_arn: String,
    pub account_name: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TagKeyValues {
    pub key: String,
    pub values: Vec<String>
}

pub struct ReportConfigManager {
    pub report_config: Vec<ReportConfig>
}

impl ReportConfigManager {
    pub fn new() -> ReportConfigManager {

        // load lambda config
        let report_config_str = include_str!("../config/report.json");
        let config: Vec<ReportConfig> = serde_json::from_str(report_config_str).unwrap();


        let manager = ReportConfigManager {
            report_config: config
        };

        manager
    }

    pub fn get_config(&self, lambda_name: &str) -> Result<ReportConfig, &str> {
        match self
            .report_config
            .clone()
            .into_iter()
            .filter(|x| x.lambda_name == lambda_name)
            .next()
        {
            Some(config) => Ok(config),
            None => Err("No matching configuration found"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn config_manager_query_by_lambda_name_tests() {
        let lambda_name = "test-lambda";
        let manager = ReportConfigManager::new();

        let config = manager.get_config(lambda_name).unwrap();

        assert_eq!(config.lambda_name, lambda_name);
        assert_eq!(config.region, "us-west-2");
        assert_eq!(
            config.cost_allocation_tags[0].key,
            "application",
        );
        assert_eq!(
            config.cost_allocation_tags[0].values[0],
            "web-app",
        );
        assert_eq!(
            config.report_query_role_arn,
            "arn:aws:iam::111111:role/cost-aware-arch-reporter"
        );
    }

    #[test]
    fn config_manager_query_by_lambda_name_err_result_tests() {
        let lambda_name = "lambda_name_does_not_exist";
        let manager = ReportConfigManager::new();

        let err = manager.get_config(lambda_name).unwrap_err();

        assert_eq!(err, "No matching configuration found");
    }
}
