use aws_sdk_s3::config::Region;
use clap::*;
use futures::join;
use serde_json::Result;
use std::time::Instant;
use util::calculator::build_date_range;
use util::csv_writer::write_to_csv;
use util::metric_client::MetricClient;
use util::cost_explorer_client::CostExplorerClient;
use util::s3_client::S3Client;

pub mod util;

#[derive(Subcommand, Debug)]
enum Command {
    /// Creates csv report
    CreateReport,
    /// Uploads csv report to S3
    UploadReport
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// AWS profile for SDK to use
    #[arg(short = 'p', long, default_value_t = String::from("default"))]
    aws_profile: String, 

    /// AWS profile for SDK to use
    #[arg(short = 'r', long, default_value_t = String::from("us-west-2"))]
    aws_region: String,

    /// Start date (YYYY-mm / e.g: 2023-04) for the report 
    #[arg(short = 's', long, default_value_t = String::from("2023-04"))]
    start_date: String,

    /// Start date (YYYY-mm / e.g: 2023-04) for the report 
    #[arg(short = 'e', long, default_value_t = String::from("2024-03"))]
    end_date: String,

    /// Name (not arn) of the lambda 
    #[arg(short, long)]
    lambda_name: String,

    /// Value for the Name AWS tag
    // ... assumed to be a cost allocation tag and therefore used by the 
    // cost explorer client when querying for costs
    #[arg(short, long)]
    tag_name: String,

    /// Example string
    #[command(subcommand)]
    command: Command
}

struct StopWatch {
    now: Instant
}
impl StopWatch {
    pub fn new() -> StopWatch{
        StopWatch {
            now: Instant::now()
        }
    }
    pub fn log_execution_duration(&self, task: &str){
        println!("{} - completed:{} ms", task, self.get_ms_duration());
    }
    fn get_ms_duration(&self) -> u128{
        self.now.elapsed().as_millis()
    }
}

#[::tokio::main]
async fn main() -> Result<()> {

    //let now = Instant::now();
    let sw = StopWatch::new();

    let args = Args::parse();
    let command = args.command;
    sw.log_execution_duration("main::command_args");

    // regardless of command - we'll need to build a csv path to either 
    // create the report, and upload it
    let csv_path = build_csv_path(args.lambda_name.as_str());
    sw.log_execution_duration("main::build_csv_path");

    match command  {
        Command::CreateReport => {
            create_report(
                &sw,
                args.aws_profile.as_str(), 
                args.aws_region.as_str(),
                csv_path.as_str(),
                args.lambda_name.as_str(), 
                args.tag_name.as_str()).await;
        }
        Command::UploadReport => {
            upload_report(
                &sw, 
                args.aws_profile.as_str(), 
                args.aws_region.as_str(), 
                &csv_path).await;
        }
    }

    sw.log_execution_duration(format!("main::{:?}", command).as_str());
    Ok(())
}

async fn create_report(sw: &StopWatch, aws_profile: &str, aws_region: &str, csv_path: &str, lambda_name: &str, lambda_tag_name: &str)
{
    let shared_config = aws_config::from_env()
        .region(Region::new(aws_region.to_string()))
        .profile_name(aws_profile)
        .load().await;
    sw.log_execution_duration("create_report::shared_config");

    let date_ranges = build_date_range("2023-04", "2024-03").unwrap();
    let metric_client = MetricClient::new(shared_config.clone());
    let ce_client = CostExplorerClient::new(shared_config);

    let mut combined_results: Vec<(&str, f64, f64, f64, f64)> = Vec::default(); 

    for date_range in &date_ranges {

        //collect futures
        let inv_resp_async = metric_client.query_for_invocations_by_date_range(date_range.0.as_str(), date_range.1.as_str(), lambda_name);
        let duration_resp_async = metric_client.query_for_duration_by_date_range(date_range.0.as_str(), date_range.1.as_str(), lambda_name);
        let cost_resp_async = ce_client.query_for_cost_by_date_range(date_range.0.as_str(), date_range.1.as_str(), lambda_tag_name);
        
        // set defaults
        let mut inv_result = f64::default(); 
        let mut duration_result = f64::default(); 
        let mut cost_result = f64::default(); 

        // go get that data, and join results....
        let join_result = join!(inv_resp_async, duration_resp_async, cost_resp_async);
        sw.log_execution_duration(format!("create_report::data_collection::{}", date_range.0).as_str());

        for metric_result in join_result.0.metric_data_results.unwrap() {
            match metric_result.values {
                Some(v) => if v.len() > 0 { inv_result = v[0]; } 
                None => println!("No values")
            }
        }
        for metric_result in join_result.1.metric_data_results.unwrap() {
            match metric_result.values {
                Some(v) => if v.len() > 0 { duration_result = v[0]; } 
                None => println!("No values")
            }
        }
        for cost_response in join_result.2.results_by_time.unwrap(){
            let total_cost: &aws_sdk_costexplorer::types::MetricValue = &cost_response.total.unwrap()["AmortizedCost"];
            cost_result = total_cost.amount.as_ref().unwrap().parse::<f64>().unwrap();
        }
        combined_results.push((date_range.0.as_str(), cost_result, inv_result, duration_result, (cost_result / inv_result) *  100000000.0));
    }

    write_to_csv(&lambda_name, csv_path, combined_results);
    sw.log_execution_duration("create_report::write_to_csv");
    sw.log_execution_duration("create_report");
}

async fn upload_report(sw: &StopWatch, aws_profile: &str, aws_region: &str, csv_path: &str)
{
    let shared_config = aws_config::from_env()
        .region(Region::new(aws_region.to_string()))
        .profile_name(aws_profile)
        .load().await;

    let s3_client = S3Client::new(shared_config);
    s3_client.upload_csv(csv_path).await;
    sw.log_execution_duration("upload_report");
}

fn build_csv_path(lambda_name :&str) -> String {
    format!("./{}-output.csv", &lambda_name)
}
