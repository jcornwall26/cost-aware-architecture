use std::str::FromStr;

use aws_config::SdkConfig;
use aws_sdk_costexplorer::operation::get_cost_and_usage::GetCostAndUsageOutput;
use aws_sdk_costexplorer::types::{
    Dimension as Cost_Dimension, DimensionValues, Expression, TagValues,
};
use aws_sdk_costexplorer::{self, types::DateInterval};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

pub struct CostExplorerClient {
    aws_client: aws_sdk_costexplorer::Client,
}

#[derive(Debug)]
struct Tag(String, String);

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
        cost_allocation_tag: &str,
    ) -> GetCostAndUsageOutput {
        let start_datetime =
            NaiveDateTime::parse_from_str(&start_time, "%Y-%m-%dT%H:%M:%SZ").unwrap();
        let start_date = start_datetime.date().to_string();
        let end_datetime = NaiveDateTime::parse_from_str(&end_time, "%Y-%m-%dT%H:%M:%SZ").unwrap();
        let end_date = end_datetime.date().to_string();
        let tag_tuple = parse_tag_key_and_value(cost_allocation_tag);

        let interval = DateInterval::builder()
            .set_start(Some(String::from(start_date)))
            .set_end(Some(String::from(end_date)))
            .build()
            .unwrap();

        let region_dimension = DimensionValues::builder()
            .set_key(Some(Cost_Dimension::Region))
            .set_values(Some(vec![String::from("us-west-2")]))
            .build();

        let tag_value: TagValues = TagValues::builder()
            .set_key(Some(String::from(tag_tuple.0)))
            .set_values(Some(vec![String::from(tag_tuple.1)]))
            .build();

        let region_exp = Expression::builder()
            .set_dimensions(Some(region_dimension))
            .build();

        let tag_exp = Expression::builder().set_tags(Some(tag_value)).build();

        let exp = Expression::builder()
            .set_and(Some(vec![region_exp, tag_exp]))
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
}

fn parse_tag_key_and_value(tag: &str) -> Tag {
    let index = match tag.find("=") {
        Some(v) => v,
        None => panic!(
            "Could not parse tag string, expected format Name=Value, provided tag argument: {}",
            tag
        ),
    };
    let name = tag.get(..index).unwrap_or_default();
    let value = tag.get(index + 1..).unwrap_or_default();
    Tag(
        String::from_str(name).unwrap(),
        String::from_str(value).unwrap(),
    )
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_tag_key_and_value_tests() {
        let tag = "Name=Value";
        let parsed_tag = parse_tag_key_and_value(tag);
        assert_eq!(parsed_tag.0, "Name");
        assert_eq!(parsed_tag.1, "Value");
    }

    #[test]
    fn parse_tag_key_and_value_with_space_tests() {
        let tag = "Name=Search and Browse - Kiwi GraphQL Server";
        let parsed_tag = parse_tag_key_and_value(tag);
        assert_eq!(parsed_tag.0, "Name");
        assert_eq!(parsed_tag.1, "Search and Browse - Kiwi GraphQL Server");
    }

    // #[test]
    // fn build_configuration_tests(){
    //     let config_str = include_str!("/config/config.json");
    //     let config : CostExplorerConfig = serde_json::from_str(config_str).unwrap();
    //     assert_eq!(config.region_dimension, "us-west-2");
    // }
}
