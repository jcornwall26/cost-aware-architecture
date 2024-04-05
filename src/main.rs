
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

use util::calculator::build_date_range;
use util::csv_writer::write_to_csv;

pub mod util;

#[::tokio::main]
async fn main() -> Result<()> {

    // parse environment variables and command line arguments
    // command line variables --> -- [lambda name] [lambda tag name]
    let args: Vec<String> = env::args().collect();
    let lambda_name = &args[1];
    let lambda_tag_name = &args[2];
    // let lambda_name = "tf-kiwi-cne-proda-shop-service";
    // let lambda_tag_name = "tf-kiwi-cne-proda-shop";

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

    let client = aws_sdk_cloudwatch::Client::new(&shared_config);
    let ce_client = aws_sdk_costexplorer::Client::new(&shared_config);

    let mut com_results: Vec<(&str, f64, f64, f64, f64)> = Vec::default(); 

    for date_range in &date_ranges {

        let inv_resp_async = query_for_invocations_by_date_range(&client, date_range.0.as_str(), date_range.1.as_str(), lambda_name).await;
        let duration_resp_async = query_for_duration_by_date_range(&client, date_range.0.as_str(), date_range.1.as_str(), lambda_name).await;
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

    // // save to CSV
    // let mut wtr = csv::Writer::from_path(format!("./{}-{}-{}-output.csv", &lambda_name, &aws_profile, &aws_region)).unwrap();

    // // header row
    // match wtr.write_record(&["function-name", "timestamp",  "monthly-cost", "invocations", "average-duration", "cost-per-100m-requests"]) {
    //     Ok(_v) => (),
    //     Err(e) => println!("{:?}", e)
    // }
    // match wtr.flush() {
    //     Ok(_v) => (),
    //     Err(_e) => ()
    // };

    // for com_result in com_results{
    //     println!("{:?}", com_result);
    //     match wtr.write_record(&[lambda_name, 
    //         com_result.0, com_result.1.to_string().as_str(), 
    //         com_result.2.to_string().as_str(), 
    //         com_result.3.to_string().as_str(), 
    //         com_result.4.to_string().as_str()]) {
    //         Ok(_v) => (),
    //         Err(e) => println!("{:?}", e)
    //     }
    //     match wtr.flush() {
    //         Ok(_v) => (),
    //         Err(_e) => ()
    //     };
    // }

    Ok(())
}

async fn query_for_invocations_by_date_range(client: &aws_sdk_cloudwatch::Client, start_time: &str, end_time: &str, lambda_name: &str) -> GetMetricDataOutput {

    println!("query_for_invocations_by_date_range::start - {} {}", start_time, end_time);

    let dimension_builder = Dimension::builder();
    let dimension: Dimension = dimension_builder
        .set_value(Some(String::from(lambda_name)))
        .set_name(Some(String::from("FunctionName"))).build();

    let inv_metric_builder = Metric::builder();
    let inv_metric = inv_metric_builder
        .set_metric_name(Some(String::from("Invocations"))) 
        .set_namespace(Some(String::from("AWS/Lambda")))
        .set_dimensions(Some(vec![dimension.clone()]))
        .build();


    // TODO - calculate correct period .....
    let inv_metric_stat_builder = MetricStat::builder();
    let inv_metric_stat = inv_metric_stat_builder
        .set_metric(Some(inv_metric))
        .set_period(Some(2678400))
        .set_stat(Some(String::from("Sum")))
        .build();

    let inv_metric_data_query_builder = MetricDataQuery::builder();
    let inv_query = inv_metric_data_query_builder
        .set_id(Some(String::from("m1")))
        .set_metric_stat(Some(inv_metric_stat))
        .set_return_data(Some(true))
        .build();

    println!("query_for_invocations_by_date_range::end - {} {}", start_time, end_time);

    client.get_metric_data()
        .set_start_time(Some(DateTime::from_str(start_time, Format::DateTime).unwrap()))
        .set_end_time(Some(DateTime::from_str(end_time, Format::DateTime).unwrap()))
        .set_metric_data_queries(Some(vec![inv_query]))
        .send().await.unwrap()
}

async fn query_for_duration_by_date_range(client: &aws_sdk_cloudwatch::Client, start_time: &str, end_time: &str, lambda_name: &str) -> GetMetricDataOutput {

    println!("query_for_duration_by_date_range::start - {} {}", start_time, end_time);

    let dimension_builder = Dimension::builder();
    let dimension: Dimension = dimension_builder
        .set_value(Some(String::from(lambda_name)))
        .set_name(Some(String::from("FunctionName"))).build();

    let duration_metric_builder = Metric::builder();
    let duration_metric = duration_metric_builder
        .set_metric_name(Some(String::from("Duration"))) 
        .set_namespace(Some(String::from("AWS/Lambda")))
        .set_dimensions(Some(vec![dimension]))
        .build();

    // TODO - calculate correct period .....
    let duration_metric_stat_builder = MetricStat::builder();
    let duration_metric_stat = duration_metric_stat_builder
        .set_metric(Some(duration_metric))
        .set_period(Some(2678400))
        .set_stat(Some(String::from("Average")))
        .build();

    let duration_metric_data_query_builder = MetricDataQuery::builder();
    let duration_query = duration_metric_data_query_builder
        .set_id(Some(String::from("m2")))
        .set_metric_stat(Some(duration_metric_stat))
        .set_return_data(Some(true))
        .build();

    println!("query_for_duration_by_date_range::end - {} {}", start_time, end_time);

    client.get_metric_data()
        .set_start_time(Some(DateTime::from_str(start_time, Format::DateTime).unwrap()))
        .set_end_time(Some(DateTime::from_str(end_time, Format::DateTime).unwrap()))
        .set_metric_data_queries(Some(vec![duration_query]))
        .send().await.unwrap()
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