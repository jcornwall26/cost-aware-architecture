
use aws_sdk_costexplorer::types::{Dimension as Cost_Dimension, DimensionValues, Expression, TagValues};
use aws_sdk_costexplorer::{self, types::DateInterval};
use aws_sdk_costexplorer::operation::get_cost_and_usage::GetCostAndUsageOutput;
use aws_sdk_cloudwatch;
use aws_sdk_cloudwatch::operation::get_metric_data::GetMetricDataOutput;
use aws_sdk_cloudwatch::types::{Dimension, Metric, MetricDataQuery, MetricStat};
use aws_sdk_s3::config::Region;
use aws_smithy_types::{date_time::Format, DateTime};
use chrono::prelude::*;
use serde_json::Result;
use std::env;

mod util;

#[::tokio::main]
async fn main() -> Result<()> {

    //load env vars 
    //todo handle missing env vars
    let aws_profile = env::var("AWS_PROFILE").unwrap();
    let aws_region = env::var("AWS_REGION").unwrap();

    let shared_config = aws_config::from_env()
        .region(Region::new(aws_region))
        .profile_name(aws_profile)
        .load().await;

    let date_ranges = [
        //("2023-01-01T00:00:00Z", "2023-01-31T00:00:00Z"),
        ("2023-02-01T00:00:00Z", "2023-02-28T00:00:00Z"),
        ("2023-03-01T00:00:00Z", "2023-03-30T00:00:00Z"),
        ("2023-04-01T00:00:00Z", "2023-04-30T00:00:00Z"),
        ("2023-05-01T00:00:00Z", "2023-05-31T00:00:00Z"),
        ("2023-06-01T00:00:00Z", "2023-06-30T00:00:00Z"),
        ("2023-07-01T00:00:00Z", "2023-07-31T00:00:00Z"),
        ("2023-08-01T00:00:00Z", "2023-08-31T00:00:00Z"),
        ("2023-09-01T00:00:00Z", "2023-09-30T00:00:00Z"),
        ("2023-10-01T00:00:00Z", "2023-10-31T00:00:00Z"),
        ("2023-11-01T00:00:00Z", "2023-11-30T00:00:00Z"),
        ("2023-12-01T00:00:00Z", "2023-12-31T00:00:00Z"),
        ("2024-01-01T00:00:00Z", "2024-01-31T00:00:00Z"),
        ("2024-02-01T00:00:00Z", "2024-02-29T00:00:00Z")
    ];

    let client = aws_sdk_cloudwatch::Client::new(&shared_config);
    let ce_client = aws_sdk_costexplorer::Client::new(&shared_config);

    let mut com_results: Vec<(f64, f64, f64)> = Vec::default();

    for date_range in date_ranges {

        let inv_resp_async = query_for_invocations_by_date_range(&client, date_range.0, date_range.1);
        let cost_resp_async = query_for_cost_by_date_range(&ce_client, date_range.0, date_range.1);
        let mut inv_result = f64::default(); 
        let mut cost_result = f64::default(); 

        for metric_result in inv_resp_async.await.metric_data_results.unwrap() {
            match metric_result.values {
                Some(v) => if v.len() > 0 { inv_result = v[0]; } 
                None => println!("No values")
            }
        }
        for cost_response in cost_resp_async.await.results_by_time.unwrap(){
            let total_cost: &aws_sdk_costexplorer::types::MetricValue = &cost_response.total.unwrap()["AmortizedCost"];
            cost_result = total_cost.amount.as_ref().unwrap().parse::<f64>().unwrap();
        }
        com_results.push((cost_result, inv_result, (cost_result / inv_result) *  100000000.0));
    }

    for com_result in com_results{
        println!("{:?}", com_result);
    }

    Ok(())
}

async fn query_for_invocations_by_date_range(client: &aws_sdk_cloudwatch::Client, start_time: &str, end_time: &str) -> GetMetricDataOutput {

    println!("query_for_invocations_by_date_range::start - {} {}", start_time, end_time);

    let dimension_builder = Dimension::builder();
    let dimension: Dimension = dimension_builder
        .set_value(Some(String::from("tf-kiwi-cne-proda-shop-service")))
        .set_name(Some(String::from("FunctionName"))).build();

    let metric_builder = Metric::builder();
    let metric = metric_builder
        .set_metric_name(Some(String::from("Invocations"))) 
        .set_namespace(Some(String::from("AWS/Lambda")))
        .set_dimensions(Some(vec![dimension]))
        .build();

    let metric_stat_builder = MetricStat::builder();
    let metric_stat = metric_stat_builder
        .set_metric(Some(metric))
        .set_period(Some(2678400))
        .set_stat(Some(String::from("Sum")))
        .build();

    let metric_data_query_builder = MetricDataQuery::builder();
    let query = metric_data_query_builder
        .set_id(Some(String::from("m1")))
        .set_metric_stat(Some(metric_stat))
        .set_return_data(Some(true))
        .build();

    println!("query_for_invocations_by_date_range::end - {} {}", start_time, end_time);

    client.get_metric_data()
        .set_start_time(Some(DateTime::from_str(start_time, Format::DateTime).unwrap()))
        .set_end_time(Some(DateTime::from_str(end_time, Format::DateTime).unwrap()))
        .metric_data_queries(query)
        .send().await.unwrap()
}

async fn query_for_cost_by_date_range(client: &aws_sdk_costexplorer::Client, start_time: &str, end_time: &str) -> GetCostAndUsageOutput {
    
    println!("query_for_cost_by_date_range::start - {} {}", start_time, end_time);

    let start_datetime = NaiveDateTime::parse_from_str(&start_time, "%Y-%m-%dT%H:%M:%SZ").unwrap();
    let start_date = start_datetime.date().to_string();
    let end_datetime = NaiveDateTime::parse_from_str(&end_time, "%Y-%m-%dT%H:%M:%SZ").unwrap();
    let end_date = end_datetime.date().to_string();
    
    let interval = DateInterval::builder()
        .set_start(Some(String::from(start_date)))
        .set_end(Some(String::from(end_date)))
        .build().unwrap(); 

    println!("query_for_cost_by_date_range::end - {} {}", start_time, end_time);

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
        .set_values(Some(vec![String::from("tf-kiwi-cne-proda-shop")]))
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

    client.get_cost_and_usage()
        .set_time_period(Some(interval))
        .set_filter(Some(exp))
        .set_metrics(Some(vec![String::from("AmortizedCost")]))
        .set_granularity(Some(aws_sdk_costexplorer::types::Granularity::Monthly))
        .send().await.unwrap()
}