use crate::graphql::pipeline::PipelinesQuery;
use anyhow::{Error as AnyhowError, Result as AnyhowResult};
use clap::{Args, Subcommand};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    process::Command,
    str, thread,
    time::{Duration, Instant},
};
use tabled::{Table, Tabled};

fn fmt_option<T: Display>(o: &Option<T>) -> String {
    match o {
        Some(s) => format!("{}", s),
        None => "N/A".to_string(),
    }
}

#[derive(Tabled, Serialize, Deserialize, Debug)]
struct PipelineData {
    name: String,
    #[tabled(display_with = "fmt_option")]
    last_succeeded_at: Option<String>,
    #[tabled(display_with = "fmt_option")]
    last_failed_at: Option<String>,
    #[tabled(display_with = "fmt_option")]
    last_duration: Option<String>,
}

#[derive(Tabled)]
struct PipelineBranch {
    name: &'static str,
    #[tabled(display_with = "fmt_option")]
    last_succeed_at: Option<&'static str>,
    #[tabled(display_with = "fmt_option")]
    last_failed_at: Option<&'static str>,
    #[tabled(display_with = "fmt_option")]
    last_duration: Option<&'static str>,
}

#[derive(Tabled)]
struct PipelinePullRequest {
    name: &'static str,
    #[tabled(display_with = "fmt_option")]
    last_succeed_at: Option<&'static str>,
    #[tabled(display_with = "fmt_option")]
    last_failed_at: Option<&'static str>,
    #[tabled(display_with = "fmt_option")]
    last_duration: Option<&'static str>,
}

#[derive(Debug, Args)]
pub struct Pipeline {
    #[clap(subcommand)]
    pub subcommand: PipelineSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum PipelineSubcommand {
    /// List the current pipelines of the application
    List,
    /// Show the details of a pipeline
    Describe {
        /// The pipeline name
        name: String,
    },
    /// Show the build status and (possible) errors on branch ci pipeline
    CiStatus,
}

impl Pipeline {
    pub async fn perform_action(&self) -> AnyhowResult<(), AnyhowError> {
        match &self.subcommand {
            PipelineSubcommand::List => {
                let started = Instant::now();
                let progress_bar = ProgressBar::new(1234);
                progress_bar.set_style(ProgressStyle::default_spinner());
                println!("Fetching pipelines list ...");

                progress_bar.inc(1);

                // Calling API ...
                let pipelines_data = PipelinesQuery::fetch()
                    .await?
                    .data
                    .ok_or(anyhow::anyhow!("Error"))?
                    .pipelines;

                progress_bar.finish_and_clear();

                println!("{:?}", pipelines_data);

                if let Some(pipelines_data) = pipelines_data {
                    let mut pipelines = Vec::new();

                    for raw_pipeline in pipelines_data {
                        let pipeline_value = serde_json::to_value(raw_pipeline)
                            .expect("Failed converting raw pipeline data to json value");
                        println!("value: {:?}", pipeline_value);
                        let pipeline: PipelineData = serde_json::from_value(pipeline_value)
                            .expect("Failed converting json value to Pipeline object");
                        pipelines.push(pipeline);
                    }

                    println!("{:#?}", pipelines);

                    let table = Table::new(pipelines).to_string();
                    println!("{table}");
                }
                println!("Fetch in {}.", HumanDuration(started.elapsed()));

                Ok(())
            }
            PipelineSubcommand::Describe { name } => {
                let deps = 1234;
                let progress_bar = ProgressBar::new(deps);
                progress_bar.set_style(ProgressStyle::default_spinner());
                println!("Fetching pipeline data ...");
                for _ in 0..deps {
                    progress_bar.inc(1);
                    thread::sleep(Duration::from_millis(3));
                }
                progress_bar.finish_and_clear();

                println!("Pipeline \"{}\":\n", name);

                let branches = vec![PipelineBranch {
                    name: "master",
                    last_succeed_at: Some("1 hr 20 min"),
                    last_failed_at: Some("6 days 19 hr"),
                    last_duration: Some("14 min"),
                }];

                let table = Table::new(branches).to_string();

                println!("Branches");
                println!("{table}");

                let pull_requests = vec![
                    PipelinePullRequest {
                        name: "PR-3985",
                        last_succeed_at: None,
                        last_failed_at: None,
                        last_duration: None,
                    },
                    PipelinePullRequest {
                        name: "PR-4037",
                        last_succeed_at: Some("1 hr 20 min"),
                        last_failed_at: Some("6 days 19 hr"),
                        last_duration: Some("14 min"),
                    },
                    PipelinePullRequest {
                        name: "PR-4086",
                        last_succeed_at: Some("1 min 2 sec"),
                        last_failed_at: None,
                        last_duration: Some("2.3 sec"),
                    },
                    PipelinePullRequest {
                        name: "PR-4096",
                        last_succeed_at: Some("1 min 2 sec"),
                        last_failed_at: None,
                        last_duration: Some("4.3 sec"),
                    },
                    PipelinePullRequest {
                        name: "PR-4113",
                        last_succeed_at: Some("18 hr"),
                        last_failed_at: None,
                        last_duration: Some("34 sec"),
                    },
                ];

                let table = Table::new(pull_requests).to_string();

                println!("Pull Requests");
                println!("{table}");

                todo!()
            }
            PipelineSubcommand::CiStatus => {
                println!("CI Status");

                let output = Command::new("git")
                    .args(["branch", "--show-current"])
                    .output()
                    .expect("failed to execute process");

                println!("{:?}", str::from_utf8(&output.stdout));

                todo!()
            }
        }
    }
}
