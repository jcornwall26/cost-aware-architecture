use aws_sdk_s3::config::Region;
use clap::*;
use futures::join;
use serde::Deserialize;
use serde_json::Result;
use std::rc::Rc;
use util::calculator::build_date_range;
use util::cost_explorer_client::CostExplorerClient;
use util::csv_writer::write_to_csv;
use util::metric_client::MetricClient;
use util::report_config::ReportConfigManager;
use util::s3_client::S3Client;
use util::stop_watch::StopWatch;

pub mod util;

#[derive(Deserialize, Subcommand, Debug)]
enum Command {
    /// Creates csv report
    CreateReport,
    /// Creates csv reports, for all defined in /config/query.json
    CreateReports,
    /// Uploads csv report to S3
    UploadReport,
    /// Uploads csv report to S3, for all defined in /config/query.json
    UploadReports,
}

#[derive(Deserialize, Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// AWS profile for SDK to use
    #[arg(short = 'p', long)]
    aws_profile: Option<String>,

    /// AWS region for SDK to use
    #[arg(short = 'r', long)]
    aws_region: Option<String>,

    /// AWS IAM role to assume when querying for report data
    #[arg(short = 'q', long)]
    report_query_role_arn: Option<String>,

    /// Start date (YYYY-mm / e.g: 2023-04) for the report
    #[arg(short = 's', long, default_value_t = String::from("2023-04"))]
    start_date: String,

    /// Start date (YYYY-mm / e.g: 2023-04) for the report
    #[arg(short = 'e', long, default_value_t = String::from("2024-03"))]
    end_date: String,

    /// Name (not arn) of the lambda
    #[arg(short, long)]
    lambda_name: Option<String>,

    /// Cost allocation tag (in the format tag-ame=tag-value e.g. Name=MyApp ) to use when querying against AWS's cost explorer
    #[arg(short = 't', long)]
    cost_allocation_tag: Option<String>,

    /// Example string
    #[command(subcommand)]
    command: Command,
}

#[::tokio::main]
async fn main() -> Result<()> {
    // start stopwatch
    let sw = Rc::new(StopWatch::new());

    // load up arguments and report config
    let args = Args::parse();
    let command = args.command;
    let report_config_manager = ReportConfigManager::new();

    match command {
        Command::CreateReport => {
            let lambda_name = match args.lambda_name {
                Some(l) => l,
                None => panic!("lambda_name arg needed for CreateReport"),
            };
            let csv_path = build_csv_path(
                &lambda_name,
                args.start_date.as_str(),
                args.end_date.as_str(),
            );

            let config = report_config_manager.get_config(&lambda_name).unwrap();
            create_report(
                Rc::clone(&sw),
                args.aws_profile.unwrap().as_str(),
                &config.region,
                &config.report_query_role_arn,
                csv_path.as_str(),
                &config.lambda_name,
                &config.cost_allocation_tag,
                args.start_date.as_str(),
                args.end_date.as_str(),
            )
            .await;
        }
        Command::CreateReports => {
            let profile = args.aws_profile.unwrap();
            for report in report_config_manager.report_config.into_iter() {
                let lambda_name = report.lambda_name;
                let csv_path = build_csv_path(
                    &lambda_name,
                    args.start_date.as_str(),
                    args.end_date.as_str(),
                );
                create_report(
                    Rc::clone(&sw),
                    profile.as_str(),
                    report.region.as_str(),
                    report.report_query_role_arn.as_str(),
                    csv_path.as_str(),
                    lambda_name.as_str(),
                    report.cost_allocation_tag.as_str(),
                    args.start_date.as_str(),
                    args.end_date.as_str(),
                )
                .await;
            }
        }
        Command::UploadReport => {
            let lambda_name = match args.lambda_name {
                Some(l) => l,
                None => panic!("lambda_name arg needed for UploadReport"),
            };
            let csv_path = build_csv_path(
                &lambda_name,
                args.start_date.as_str(),
                args.end_date.as_str(),
            );
            upload_report(
                Rc::clone(&sw),
                args.aws_profile.unwrap().as_str(),
                args.aws_region.unwrap().as_str(),
                &csv_path,
            )
            .await;
        }
        Command::UploadReports => {
            let profile = args.aws_profile.unwrap();
            let region = args.aws_region.unwrap();

            for report in report_config_manager.report_config.into_iter() {
                let csv_path = build_csv_path(
                    &report.lambda_name,
                    args.start_date.as_str(),
                    args.end_date.as_str(),
                );

                upload_report(Rc::clone(&sw), &profile, &region, &csv_path).await;
            }
        }
    }

    sw.log_execution_duration(format!("main::{:?}", command).as_str());

    Ok(())
}

async fn create_report(
    sw: Rc<StopWatch>,
    aws_profile: &str,
    aws_region: &str,
    report_query_role_arn: &str,
    csv_path: &str,
    lambda_name: &str,
    lambda_tag_name: &str,
    start_date: &str,
    end_date: &str,
) {
    let local_config = aws_config::from_env()
        .region(Region::new(aws_region.to_string()))
        .profile_name(aws_profile)
        .load()
        .await;

    let provider = aws_config::sts::AssumeRoleProvider::builder(report_query_role_arn)
        .configure(&local_config)
        .build()
        .await;

    let assumed_config = aws_config::from_env()
        .credentials_provider(provider)
        .region(Region::new(aws_region.to_string()))
        .load()
        .await;

    sw.log_execution_duration("create_report::shared_config");

    let date_ranges = build_date_range(start_date, end_date).unwrap();
    let metric_client = MetricClient::new(assumed_config.clone());
    let ce_client = CostExplorerClient::new(assumed_config);

    let mut combined_results: Vec<(&str, f64, f64, f64, f64)> = Vec::default();

    for date_range in &date_ranges {
        //collect futures
        let inv_resp_async = metric_client.query_for_invocations_by_date_range(
            date_range.0.as_str(),
            date_range.1.as_str(),
            lambda_name,
        );
        let duration_resp_async = metric_client.query_for_duration_by_date_range(
            date_range.0.as_str(),
            date_range.1.as_str(),
            lambda_name,
        );
        let cost_resp_async = ce_client.query_for_cost_by_date_range(
            date_range.0.as_str(),
            date_range.1.as_str(),
            lambda_tag_name,
        );

        // set defaults
        let mut inv_result = f64::default();
        let mut duration_result = f64::default();
        let mut cost_result = f64::default();

        // go get that data, and join results....
        let join_result = join!(inv_resp_async, duration_resp_async, cost_resp_async);
        sw.log_execution_duration(
            format!("create_report::data_collection::{}", date_range.0).as_str(),
        );

        for metric_result in join_result.0.metric_data_results.unwrap() {
            match metric_result.values {
                Some(v) => {
                    if v.len() > 0 {
                        inv_result = v[0];
                    }
                }
                None => println!("No values"),
            }
        }
        for metric_result in join_result.1.metric_data_results.unwrap() {
            match metric_result.values {
                Some(v) => {
                    if v.len() > 0 {
                        duration_result = v[0];
                    }
                }
                None => println!("No values"),
            }
        }
        for cost_response in join_result.2.results_by_time.unwrap() {
            let total_cost: &aws_sdk_costexplorer::types::MetricValue =
                &cost_response.total.unwrap()["AmortizedCost"];
            cost_result = total_cost.amount.as_ref().unwrap().parse::<f64>().unwrap();
        }
        combined_results.push((
            date_range.0.as_str(),
            cost_result,
            inv_result,
            duration_result,
            (cost_result / inv_result) * 100000000.0,
        ));
    }

    write_to_csv(&lambda_name, csv_path, combined_results);
    sw.log_execution_duration("create_report::write_to_csv");
    sw.log_execution_duration("create_report");
}

async fn upload_report(sw: Rc<StopWatch>, aws_profile: &str, aws_region: &str, csv_path: &str) {
    let shared_config = aws_config::from_env()
        .region(Region::new(aws_region.to_string()))
        .profile_name(aws_profile)
        .load()
        .await;

    let s3_client = S3Client::new(shared_config);
    s3_client.upload_csv(csv_path).await;
    sw.log_execution_duration("upload_report");
}

fn build_csv_path(lambda_name: &str, start_date: &str, end_date: &str) -> String {
    format!("./{}-{}-{}-output.csv", &lambda_name, start_date, end_date)
}
