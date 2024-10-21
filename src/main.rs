use clap::*;
use report_manager::{create_report, create_report_account, upload_report, upload_report_account};
use serde::Deserialize;
use serde_json::Result;
use std::rc::Rc;
use util::report_config::ReportConfigManager;
use util::stop_watch::StopWatch;

pub mod report_manager;
pub mod util;

#[derive(Deserialize, Subcommand, Debug)]
enum Command {
    CreateReport {
        /// Name (not arn) of the lambda
        #[arg(short, long)]
        lambda_name: String,
    },
    /// Creates csv reports, for all defined in /config/query.json
    CreateReports,
    // Creates a csv report, for the entire account
    CreateReportAccount,
    /// Uploads csv report to S3
    UploadReport {
        /// Name (not arn) of the lambda
        #[arg(short, long)]
        lambda_name: String,
        /// AWS region for SDK to use
        #[arg(short = 'r', long)]
        aws_region: String,
    },
    /// Uploads csv report to S3, for all defined in /config/query.json
    UploadReports {
        /// AWS region for SDK to use
        #[arg(short = 'r', long)]
        aws_region: String,
    },
    // Uploads csv report to s3, for the entire account
    UploadReportAccount {
        /// AWS region for SDK to use
        #[arg(short = 'r', long)]
        aws_region: String,
    },
}

#[derive(Deserialize, Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// AWS profile for SDK to use
    #[arg(short = 'p', long)]
    aws_profile: String,

    /// Flag to drive is the role defined in config should be assumed
    #[arg(short = 'a', long, default_value_t = String::from("true"))]
    aws_assume_role: String,

    /// Start date (YYYY-mm / e.g: 2023-04) for the report
    #[arg(short = 's', long)]
    start_date: String,

    /// Start date (YYYY-mm / e.g: 2023-04) for the report
    #[arg(short = 'e', long)]
    end_date: String,

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
        Command::CreateReport { lambda_name } => {
            let csv_path = build_csv_path(
                &lambda_name,
                args.start_date.as_str(),
                args.end_date.as_str(),
            );

            let config = report_config_manager.get_config(&lambda_name).unwrap();
            create_report(
                Rc::clone(&sw),
                args.aws_profile.as_str(),
                args.aws_assume_role.parse::<bool>().unwrap(),
                args.start_date.as_str(),
                args.end_date.as_str(),
                csv_path.as_str(),
                &config.lambda_name,
                &config.region,
                &config.report_query_role_arn,
                config.cost_allocation_tags,
            )
            .await;
        }
        Command::CreateReports => {
            for report in report_config_manager.report_config.into_iter() {
                let lambda_name = report.lambda_name;
                let csv_path = build_csv_path(
                    &lambda_name,
                    args.start_date.as_str(),
                    args.end_date.as_str(),
                );
                create_report(
                    Rc::clone(&sw),
                    args.aws_profile.as_str(),
                    args.aws_assume_role.parse::<bool>().unwrap(),
                    args.start_date.as_str(),
                    args.end_date.as_str(),
                    csv_path.as_str(),
                    lambda_name.as_str(),
                    report.region.as_str(),
                    report.report_query_role_arn.as_str(),
                    report.cost_allocation_tags,
                )
                .await;
            }
        }
        Command::UploadReport {
            lambda_name,
            aws_region,
        } => {
            let csv_path = build_csv_path(
                &lambda_name,
                args.start_date.as_str(),
                args.end_date.as_str(),
            );
            upload_report(
                Rc::clone(&sw),
                args.aws_profile.as_str(),
                &aws_region,
                &csv_path,
            )
            .await;
        }
        Command::UploadReports { aws_region } => {
            let profile = args.aws_profile;

            for report in report_config_manager.report_config.into_iter() {
                let csv_path = build_csv_path(
                    &report.lambda_name,
                    args.start_date.as_str(),
                    args.end_date.as_str(),
                );

                upload_report(Rc::clone(&sw), &profile, &aws_region, &csv_path).await;
            }
        }
        Command::CreateReportAccount => {
            let csv_path = build_csv_path(
                "account_test",
                args.start_date.as_str(),
                args.end_date.as_str(),
            );

            // todo add account config
            create_report_account(
                Rc::clone(&sw),
                args.aws_profile.as_str(),
                args.aws_assume_role.parse::<bool>().unwrap(),
                args.start_date.as_str(),
                args.end_date.as_str(),
                csv_path.as_str(),
                "us-west-2",
                "arn:aws:iam::048430863637:role/lll-cost-aware-arch-reporter",
            )
            .await;
        }
        Command::UploadReportAccount { aws_region } => {
            let profile = args.aws_profile;

            let csv_path = build_csv_path(
                "account_test",
                args.start_date.as_str(),
                args.end_date.as_str(),
            );

            upload_report_account(Rc::clone(&sw), &profile, &aws_region, &csv_path).await;
        }
    }

    Ok(())
}

fn build_csv_path(lambda_name: &str, start_date: &str, end_date: &str) -> String {
    format!("./{}-{}-{}-output.csv", &lambda_name, start_date, end_date)
}
