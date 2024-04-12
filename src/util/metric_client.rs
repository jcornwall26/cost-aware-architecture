use aws_config::SdkConfig;
use aws_smithy_types::{date_time::Format, DateTime};
use aws_sdk_cloudwatch::types::{Dimension, Metric, MetricDataQuery, MetricStat};
use aws_sdk_cloudwatch::operation::get_metric_data::GetMetricDataOutput;

pub struct MetricClient {
    aws_client: aws_sdk_cloudwatch::Client
}

impl MetricClient {
    pub fn new(shared_config: SdkConfig) -> MetricClient {
        MetricClient {
            aws_client: aws_sdk_cloudwatch::Client::new(&shared_config)
        }
    }

    pub async fn query_for_invocations_by_date_range(&self, start_time: &str, end_time: &str, lambda_name: &str) -> GetMetricDataOutput{
        println!("query_for_invocations_by_date_range::start - {} {}", start_time, end_time);
        let output = self.generic_query(start_time, end_time, lambda_name, "Invocations", "Sum").await;
        println!("query_for_invocations_by_date_range::end - {} {}", start_time, end_time);
        output
    }

    pub async fn query_for_duration_by_date_range(&self, start_time: &str, end_time: &str, lambda_name: &str) -> GetMetricDataOutput{
        println!("query_for_duration_by_date_range::start - {} {}", start_time, end_time);
        let output = self.generic_query(start_time, end_time, lambda_name, "Duration", "Average").await;
        println!("query_for_duration_by_date_range::end - {} {}", start_time, end_time);
        output
    }

    async fn generic_query(&self, start_time: &str, end_time: &str, lambda_name: &str, metric_name: &str, metric_stat: &str) -> GetMetricDataOutput
    {
        let dimension_builder = Dimension::builder();
        let dimension: Dimension = dimension_builder
            .set_value(Some(String::from(lambda_name)))
            .set_name(Some(String::from("FunctionName"))).build();
    
        let metric_builder = Metric::builder();
        let metric = metric_builder
            .set_metric_name(Some(String::from(metric_name))) 
            .set_namespace(Some(String::from("AWS/Lambda")))
            .set_dimensions(Some(vec![dimension.clone()]))
            .build();
    
        // TODO - calculate correct period .....
        let metric_stat_builder = MetricStat::builder();
        let metric_stat = metric_stat_builder
            .set_metric(Some(metric))
            .set_period(Some(2678400))
            .set_stat(Some(String::from(metric_stat)))
            .build();
    
        let metric_data_query_builder = MetricDataQuery::builder();
        let query = metric_data_query_builder
            .set_id(Some(String::from("m1")))
            .set_metric_stat(Some(metric_stat))
            .set_return_data(Some(true))
            .build();
    
        self.aws_client.get_metric_data()
            .set_start_time(Some(DateTime::from_str(start_time, Format::DateTime).unwrap()))
            .set_end_time(Some(DateTime::from_str(end_time, Format::DateTime).unwrap()))
            .set_metric_data_queries(Some(vec![query]))
            .send().await.unwrap()
    }
}