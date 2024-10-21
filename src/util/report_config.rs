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
    pub report_config: Vec<ReportConfig>,
    pub account_report_config: Vec<AccountReportConfig>,
}

impl ReportConfigManager {
    pub fn new() -> ReportConfigManager {

        // load lambda config
        let report_config_str = include_str!("../config/report.json");
        let config: Vec<ReportConfig> = serde_json::from_str(report_config_str).unwrap();

        // load lambda config
        let account_report_config_str = include_str!("../config/account_report.json");
        let account_config: Vec<AccountReportConfig> = serde_json::from_str(account_report_config_str).unwrap();

        let manager = ReportConfigManager {
            report_config: config,
            account_report_config: account_config
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

    pub fn get_account_config(&self, account_name: &str) -> Result<AccountReportConfig, &str> {
        match self
            .account_report_config
            .clone()
            .into_iter()
            .filter(|x| x.account_name == account_name)
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
        let lambda_name = "tf-ecom-web-app-prodb-shop-service";
        let manager = ReportConfigManager::new();

        let config = manager.get_config(lambda_name).unwrap();

        assert_eq!(config.lambda_name, lambda_name);
        assert_eq!(config.region, "us-west-2");
        assert_eq!(
            config.cost_allocation_tags[0].key,
            "lll:business:application-name",
        );
        assert_eq!(
            config.cost_allocation_tags[0].values[0],
            "web-app",
        );
        assert_eq!(
            config.cost_allocation_tags[0].values[1],
            "ecom-web-app",
        );
        assert_eq!(
            config.report_query_role_arn,
            "arn:aws:iam::048430863637:role/lll-cost-aware-arch-reporter"
        );
    }

    #[test]
    fn config_manager_query_by_lambda_name_err_result_tests() {
        let lambda_name = "lambda_name_does_not_exist";
        let manager = ReportConfigManager::new();

        let err = manager.get_config(lambda_name).unwrap_err();

        assert_eq!(err, "No matching configuration found");
    }

    #[test]
    fn config_manager_query_account_by_name_tests() {
        let name = "content";
        let manager = ReportConfigManager::new();

        let config = manager.get_account_config(name).unwrap();

        assert_eq!(config.account_name, name);
        assert_eq!(config.region, "us-west-2");
        assert_eq!(
            config.report_query_role_arn,
            "arn:aws:iam::048430863637:role/lll-cost-aware-arch-reporter"
        );
    }

    #[test]
    fn config_manager_query_account_by_name_err_result_tests() {
        let lambda_name = "account_name_does_not_exist";
        let manager = ReportConfigManager::new();

        let err = manager.get_account_config(lambda_name).unwrap_err();

        assert_eq!(err, "No matching configuration found");
    }
}
