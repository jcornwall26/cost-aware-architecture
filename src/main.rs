use aws_sdk_s3::config::Region;
use clap::*;
use serde_json::Result;
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

#[::tokio::main]
async fn main() -> Result<()> {

    let args = Args::parse();
    let command = args.command;

    // regardless of command - we'll need to build a csv path to either 
    // create the report, and upload it
    let csv_path = build_csv_path(args.lambda_name.as_str());

    match command  {
        Command::CreateReport => {
            create_report(args.aws_profile.as_str(), 
                args.aws_region.as_str(),
                csv_path.as_str(),
                args.lambda_name.as_str(), 
                args.tag_name.as_str()).await;
        }
        Command::UploadReport => {
            upload_report(
                args.aws_profile.as_str(), 
                args.aws_region.as_str(), 
                &csv_path).await;
        }
    }

    Ok(())
}

async fn create_report(aws_profile: &str, aws_region: &str, csv_path: &str, lambda_name: &str, lambda_tag_name: &str)
{
    let shared_config = aws_config::from_env()
        .region(Region::new(aws_region.to_string()))
        .profile_name(aws_profile)
        .load().await;

    let date_ranges = build_date_range("2023-04", "2024-03").unwrap();
    let metric_client = MetricClient::new(shared_config.clone());
    let ce_client = CostExplorerClient::new(shared_config);

    let mut combined_results: Vec<(&str, f64, f64, f64, f64)> = Vec::default(); 

    for date_range in &date_ranges {

        let inv_resp_async = metric_client.query_for_invocations_by_date_range(date_range.0.as_str(), date_range.1.as_str(), lambda_name).await;
        let duration_resp_async = metric_client.query_for_duration_by_date_range(date_range.0.as_str(), date_range.1.as_str(), lambda_name).await;
        let cost_resp_async = ce_client.query_for_cost_by_date_range(date_range.0.as_str(), date_range.1.as_str(), lambda_tag_name).await;
        let mut inv_result = f64::default(); 
        let mut duration_result = f64::default(); 
        let mut cost_result = f64::default(); 

        //tokio::join!()

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
        combined_results.push((date_range.0.as_str(), cost_result, inv_result, duration_result, (cost_result / inv_result) *  100000000.0));
    }

    write_to_csv(&lambda_name, csv_path, combined_results);
}

async fn upload_report(aws_profile: &str, aws_region: &str, csv_path: &str)
{
    let shared_config = aws_config::from_env()
        .region(Region::new(aws_region.to_string()))
        .profile_name(aws_profile)
        .load().await;

    let s3_client = S3Client::new(shared_config);
    s3_client.upload_csv(csv_path).await
}

fn build_csv_path(lambda_name :&str) -> String {
    format!("./{}-output.csv", &lambda_name)
}
