
use aws_sdk_costexplorer::types::{Dimension as Cost_Dimension, DimensionValues, Expression, TagValues};
use aws_sdk_costexplorer::{self, types::DateInterval};
use aws_sdk_costexplorer::operation::get_cost_and_usage::GetCostAndUsageOutput;
use aws_sdk_s3::config::Region;
use chrono::prelude::*;
use serde_json::Result;
use std::env;

use util::calculator::build_date_range;
use util::csv_writer::write_to_csv;
use util::metric_client::MetricClient;

pub mod util;

#[::tokio::main]
async fn main() -> Result<()> {

    // parse environment variables and command line arguments
    // command line variables --> -- [lambda name] [lambda tag name]
    let args: Vec<String> = env::args().collect();
    let lambda_name = &args[1];
    let lambda_tag_name = &args[2];

    let aws_profile = match env::var("AWS_PROFILE") {
        Ok(v) => v,
        Err(_e) => String::from("cne-prd")
    };

    let aws_region: String = match env::var("AWS_REGION") {
        Ok(v) => v,
        Err(_e) => String::from("us-west-2")
    };

    let shared_config = aws_config::from_env()
        .region(Region::new(aws_region.clone()))
        .profile_name(&aws_profile)
        .load().await;

    let date_ranges = build_date_range("2023-04", "2024-03").unwrap();

    let metric_client = MetricClient::new(shared_config.clone());

    let ce_client = aws_sdk_costexplorer::Client::new(&shared_config);

    let mut com_results: Vec<(&str, f64, f64, f64, f64)> = Vec::default(); 

    for date_range in &date_ranges {

        let inv_resp_async = metric_client.query_for_invocations_by_date_range(date_range.0.as_str(), date_range.1.as_str(), lambda_name).await;
        let duration_resp_async = metric_client.query_for_duration_by_date_range(date_range.0.as_str(), date_range.1.as_str(), lambda_name).await;
        let cost_resp_async = query_for_cost_by_date_range(&ce_client, date_range.0.as_str(), date_range.1.as_str(), lambda_tag_name).await;
        let mut inv_result = f64::default(); 
        let mut duration_result = f64::default(); 
        let mut cost_result = f64::default(); 

        for metric_result in inv_resp_async.metric_data_results.unwrap() {
            match metric_result.values {
                Some(v) => if v.len() > 0 { inv_result = v[0]; } 
                None => println!("No values")
            }
        }
        for metric_result in duration_resp_async.metric_data_results.unwrap() {
            match metric_result.values {
                Some(v) => if v.len() > 0 { duration_result = v[0]; } 
                None => println!("No values")
            }
        }
        for cost_response in cost_resp_async.results_by_time.unwrap(){
            let total_cost: &aws_sdk_costexplorer::types::MetricValue = &cost_response.total.unwrap()["AmortizedCost"];
            cost_result = total_cost.amount.as_ref().unwrap().parse::<f64>().unwrap();
        }
        com_results.push((date_range.0.as_str(), cost_result, inv_result, duration_result, (cost_result / inv_result) *  100000000.0));
    }

    write_to_csv(lambda_name, &aws_profile, &aws_region, com_results);

    Ok(())
}

async fn query_for_cost_by_date_range(client: &aws_sdk_costexplorer::Client, start_time: &str, end_time: &str, lambda_tag_name: &str) -> GetCostAndUsageOutput {
    
    println!("query_for_cost_by_date_range::start - {} {}", start_time, end_time);

    let start_datetime = NaiveDateTime::parse_from_str(&start_time, "%Y-%m-%dT%H:%M:%SZ").unwrap();
    let start_date = start_datetime.date().to_string();
    let end_datetime = NaiveDateTime::parse_from_str(&end_time, "%Y-%m-%dT%H:%M:%SZ").unwrap();
    let end_date = end_datetime.date().to_string();
    
    let interval = DateInterval::builder()
        .set_start(Some(String::from(start_date)))
        .set_end(Some(String::from(end_date)))
        .build().unwrap(); 

    let region_dimension = DimensionValues::builder()
        .set_key(Some(Cost_Dimension::Region))
        .set_values(Some(vec![String::from("us-west-2")]))
        .build();

    let service_dimension = DimensionValues::builder()
        .set_key(Some(Cost_Dimension::Service))
        .set_values(Some(vec![String::from("AWS Lambda")]))
        .build();

    let tag_value : TagValues = TagValues::builder()
        .set_key(Some(String::from("Name")))
        .set_values(Some(vec![String::from(lambda_tag_name)]))
        .build();

    let region_exp = Expression::builder()
        .set_dimensions(Some(region_dimension))
        .build();

    let service_exp = Expression::builder()
        .set_dimensions(Some(service_dimension))
        .build();

    let tag_exp = Expression::builder()
        .set_tags(Some(tag_value))
        .build();

    let exp = Expression::builder()
        .set_and(Some(vec![region_exp,service_exp, tag_exp]))
        .build();

    println!("query_for_cost_by_date_range::end - {} {}", start_time, end_time);

    client.get_cost_and_usage()
        .set_time_period(Some(interval))
        .set_filter(Some(exp))
        .set_metrics(Some(vec![String::from("AmortizedCost")]))
        .set_granularity(Some(aws_sdk_costexplorer::types::Granularity::Monthly))
        .send().await.unwrap()
}