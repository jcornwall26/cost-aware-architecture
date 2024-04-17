use aws_config::SdkConfig;
use aws_sdk_costexplorer::types::{Dimension as Cost_Dimension, DimensionValues, Expression, TagValues};
use aws_sdk_costexplorer::{self, types::DateInterval};
use aws_sdk_costexplorer::operation::get_cost_and_usage::GetCostAndUsageOutput;
use chrono::prelude::*;

pub struct CostExplorerClient {
    aws_client: aws_sdk_costexplorer::Client
}

impl CostExplorerClient {
    pub fn new(shared_config: SdkConfig) -> CostExplorerClient {
        CostExplorerClient {
            aws_client: aws_sdk_costexplorer::Client::new(&shared_config)
        }
    }

    pub async fn query_for_cost_by_date_range(&self, start_time: &str, end_time: &str, lambda_tag_name: &str) -> GetCostAndUsageOutput {
    
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
    
        self.aws_client.get_cost_and_usage()
            .set_time_period(Some(interval))
            .set_filter(Some(exp))
            .set_metrics(Some(vec![String::from("AmortizedCost")]))
            .set_granularity(Some(aws_sdk_costexplorer::types::Granularity::Monthly))
            .send().await.unwrap()
    }
}