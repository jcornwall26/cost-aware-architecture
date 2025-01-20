use clap::*;
use report_manager::create_report;
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
    }
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
            let config = report_config_manager.get_config(&lambda_name).unwrap();
            create_report(
                Rc::clone(&sw),
                args.aws_profile.as_str(),
                args.aws_assume_role.parse::<bool>().unwrap(),
                args.start_date.as_str(),
                args.end_date.as_str(),
                &config.lambda_name,
                &config.region,
                &config.report_query_role_arn,
                config.cost_allocation_tags,
            )
            .await;
        }
    }

    Ok(())
}
