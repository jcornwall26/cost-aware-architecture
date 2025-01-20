use std::rc::Rc;

use aws_config::SdkConfig;
use aws_sdk_costexplorer::operation::get_cost_and_usage::GetCostAndUsageOutput;
use aws_sdk_costexplorer::types::{
    Dimension as Cost_Dimension, DimensionValues, Expression, TagValues,
};
use aws_sdk_costexplorer::{self, types::DateInterval};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use super::report_config::TagKeyValues;

pub struct CostExplorerClient {
    aws_client: aws_sdk_costexplorer::Client,
}

#[derive(Serialize, Deserialize)]
struct CostExplorerConfig {
    region_dimension: String,
    cost_metric: String,
}

impl CostExplorerClient {
    pub fn new(shared_config: SdkConfig) -> CostExplorerClient {
        CostExplorerClient {
            aws_client: aws_sdk_costexplorer::Client::new(&shared_config),
        }
    }

    pub async fn query_for_cost_by_date_range(
        &self,
        start_time: &str,
        end_time: &str,
        cost_allocation_tags: Option<Rc<Vec<TagKeyValues>>>) -> GetCostAndUsageOutput {
        let start_datetime =
            NaiveDateTime::parse_from_str(&start_time, "%Y-%m-%dT%H:%M:%SZ").unwrap();
        let start_date = start_datetime.date().to_string();
        let end_datetime = NaiveDateTime::parse_from_str(&end_time, "%Y-%m-%dT%H:%M:%SZ").unwrap();
        let end_date = end_datetime.date().to_string();

        let interval = DateInterval::builder()
            .set_start(Some(String::from(start_date)))
            .set_end(Some(String::from(end_date)))
            .build()
            .unwrap();

        let region_dimension = DimensionValues::builder()
            .set_key(Some(Cost_Dimension::Region))
            .set_values(Some(vec![String::from("us-west-2")]))
            .build();

        let mut expressions : Vec<Expression> = Vec::new();

        match cost_allocation_tags {
            Some(tags) => {
                println!("Size of allocation tags {}", tags.len());
                for tag in tags.iter() {
                        let tag_value: TagValues = TagValues::builder()
                            .set_key(Some(String::from(tag.key.as_str())))
                            .set_values(Some(tag.values.clone()))
                            .build();
                        let tag_exp = Expression::builder().set_tags(Some(tag_value)).build();
                        println!("{:?}", tag_exp);
                        expressions.push(tag_exp);
                    }

                    expressions.push(Expression::builder()
                        .set_dimensions(Some(region_dimension))
                        .build());
            }
            None => println!("No tags, therefore no expressions")
        }
            
        let exp = Expression::builder()
            .set_and(Some(expressions))
            .build();

        self.aws_client
            .get_cost_and_usage()
            .set_time_period(Some(interval))
            .set_filter(Some(exp))
            .set_metrics(Some(vec![String::from("AmortizedCost")]))
            .set_granularity(Some(aws_sdk_costexplorer::types::Granularity::Monthly))
            .send()
            .await
            .unwrap()
    }

    pub async fn query_for_account_cost_by_date_range(
        &self,
        start_time: &str,
        end_time: &str) -> GetCostAndUsageOutput {
        let start_datetime =
            NaiveDateTime::parse_from_str(&start_time, "%Y-%m-%dT%H:%M:%SZ").unwrap();
        let start_date = start_datetime.date().to_string();
        let end_datetime = NaiveDateTime::parse_from_str(&end_time, "%Y-%m-%dT%H:%M:%SZ").unwrap();
        let end_date = end_datetime.date().to_string();

        let interval = DateInterval::builder()
            .set_start(Some(String::from(start_date)))
            .set_end(Some(String::from(end_date)))
            .build()
            .unwrap();

        self.aws_client
            .get_cost_and_usage()
            .set_time_period(Some(interval))
            .set_metrics(Some(vec![String::from("AmortizedCost")]))
            .set_granularity(Some(aws_sdk_costexplorer::types::Granularity::Monthly))
            .send()
            .await
            .unwrap()
    }
}

