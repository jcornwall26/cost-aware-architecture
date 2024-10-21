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

// #[derive(Debug)]
// struct Tag(String, String);

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

// fn parse_tag_key_and_value_list(tag_strs: &Vec<String>) -> Vec<Tag> {

//     let mut tags: Vec<Tag> = Vec::new();
//     for tag_str in tag_strs {

//         let index = match tag_str.find("=") {
//             Some(v) => v,
//             None => panic!(
//                 "Could not parse tag string, expected format Name=Value, provided tag argument: {}",
//                 tag_str
//             ),
//         };
//         let name = tag_str.get(..index).unwrap_or_default();
//         let value = tag_str.get(index + 1..).unwrap_or_default();
//         tags.push(Tag(
//             String::from_str(name).unwrap(),
//             String::from_str(value).unwrap(),
//         ));
//     }
//     tags
// }

#[cfg(test)]
mod tests {

    // use super::*;

    // #[test]
    // fn parse_tag_key_and_value_tests() {
    //     let tags = vec![String::from("Name=Value")];
    //     let parsed_tag = parse_tag_key_and_value_list(&tags);
    //     assert_eq!(parsed_tag[0].0, "Name");
    //     assert_eq!(parsed_tag[0].1, "Value");
    // }

    // #[test]
    // fn parse_tag_key_and_value_with_space_tests() {
    //     let tag = vec![String::from("Name=Search and Browse - Kiwi GraphQL Server")];
    //     let parsed_tag = parse_tag_key_and_value_list(&tag);
    //     assert_eq!(parsed_tag[0].0, "Name");
    //     assert_eq!(parsed_tag[0].1, "Search and Browse - Kiwi GraphQL Server");
    // }

    // #[test]
    // fn build_configuration_tests(){
    //     let config_str = include_str!("/config/config.json");
    //     let config : CostExplorerConfig = serde_json::from_str(config_str).unwrap();
    //     assert_eq!(config.region_dimension, "us-west-2");
    // }
}
