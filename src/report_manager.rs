use aws_sdk_s3::config::Region;
use futures::join;
use std::rc::Rc;
use crate::util::calculator::build_date_range;
use crate::util::cost_explorer_client::CostExplorerClient;
use crate::util::metric_client::MetricClient;
use crate::util::report_config::TagKeyValues;
use crate::util::opensearch_client::OpenSearchClient;
use crate::util::stop_watch::StopWatch;

pub async fn create_report(
    sw: Rc<StopWatch>,
    aws_profile: &str,
    aws_assume_role: bool,
    start_date: &str,
    end_date: &str,
    lambda_name: &str,
    aws_region: &str,
    report_query_role_arn: &str,
    cost_allocation_tags: Vec<TagKeyValues>,
) {
    let profile_config = load_profile_config(aws_region, aws_profile, aws_assume_role, report_query_role_arn).await;

    sw.log_execution_duration("create_report::load_profile_config");

    let date_ranges = build_date_range(start_date, end_date).unwrap();
    let metric_client = MetricClient::new(profile_config.clone());
    let ce_client = CostExplorerClient::new(profile_config);
    let rc_tags = Rc::new(cost_allocation_tags);

    let mut combined_results: Vec<(&str, f64, f64, f64, f64)> = Vec::default();

    for date_range in &date_ranges {
        //collect futures
        let inv_resp_async = metric_client.query_for_invocations_by_date_range(
            date_range.0.as_str(),
            date_range.1.as_str(),
            lambda_name,
        );
        let duration_resp_async = metric_client.query_for_duration_by_date_range(
            date_range.0.as_str(),
            date_range.1.as_str(),
            lambda_name,
        );
        let cost_resp_async = ce_client.query_for_cost_by_date_range(
            date_range.0.as_str(),
            date_range.1.as_str(),
            Some(Rc::clone(&rc_tags)),
        );

        // set defaults
        let mut inv_result = f64::default();
        let mut duration_result = f64::default();
        let mut cost_result = f64::default();

        // go get that data, and join results....
        let join_result = join!(inv_resp_async, duration_resp_async, cost_resp_async);
        sw.log_execution_duration(
            format!("create_report::data_collection::{}", date_range.0).as_str(),
        );

        for metric_result in join_result.0.metric_data_results.unwrap() {
            match metric_result.values {
                Some(v) => {
                    if v.len() > 0 {
                        inv_result = v[0];
                    }
                }
                None => println!("No values"),
            }
        }
        for metric_result in join_result.1.metric_data_results.unwrap() {
            match metric_result.values {
                Some(v) => {
                    if v.len() > 0 {
                        duration_result = v[0];
                    }
                }
                None => println!("No values"),
            }
        }
        for cost_response in join_result.2.results_by_time.unwrap() {
            let total_cost: &aws_sdk_costexplorer::types::MetricValue =
                &cost_response.total.unwrap()["AmortizedCost"];
            cost_result = total_cost.amount.as_ref().unwrap().parse::<f64>().unwrap();
        }
        combined_results.push((
            date_range.0.as_str(),
            cost_result,
            inv_result,
            duration_result,
            (cost_result / inv_result) * 100000000.0,
        ));
    }

    let os_client = OpenSearchClient::new();
    os_client.write_to_opensearch(&lambda_name, combined_results).await;
    sw.log_execution_duration("create_report::write_to_opensearch");
    sw.log_execution_duration("create_report");
}

async fn load_profile_config(aws_region: &str, aws_profile: &str, aws_assume_role: bool, report_query_role_arn: &str) -> aws_config::SdkConfig {
    
    let local_config = aws_config::from_env()
    .region(Region::new(aws_region.to_string()))
    .profile_name(aws_profile)
    .load()
    .await;

    match aws_assume_role {
        true => {
            let provider = aws_config::sts::AssumeRoleProvider::builder(report_query_role_arn)
                .configure(&local_config)
                .build()
                .await;

            aws_config::from_env()
                .credentials_provider(provider)
                .region(Region::new(aws_region.to_string()))
                .load()
                .await
        }
        false => local_config,
    }
}