use crate::config::KurecConfig;
use clap::{Parser, Subcommand};

pub mod adapter;
mod cmd;
mod config;
mod domain;
mod message;
mod model;

#[derive(Clone, Debug, Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = env!("CARGO_PKG_DESCRIPTION"))]
struct Cli {
    #[clap(subcommand)]
    subcommand: SubCommands,
}

#[derive(Clone, Debug, Subcommand)]
enum SubCommands {
    SseConverter {
        #[clap(long = "proto", default_value = "http")]
        protocol: String,
        #[clap(long = "host", default_value = "tuner")]
        tuner_host: String,
        #[clap(long = "port", default_value = "40772")]
        tuner_port: u16,
    },
    SseEpg {
        #[clap(long = "proto", default_value = "http")]
        protocol: String,
        #[clap(long = "host", default_value = "tuner")]
        tuner_host: String,
        #[clap(long = "port", default_value = "40772")]
        tuner_port: u16,
    },
    SseRecord {
        #[clap(long = "proto", default_value = "http")]
        protocol: String,
        #[clap(long = "host", default_value = "tuner")]
        tuner_host: String,
        #[clap(long = "port", default_value = "40772")]
        tuner_port: u16,
    },
    RuleMeilisearch {},
    StreamRecord {},
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = KurecConfig::get_config()?;

    match cli.subcommand {
        SubCommands::SseConverter {
            protocol,
            tuner_host,
            tuner_port,
        } => {
            let tuner_url = format!("{}://{}:{}", protocol, tuner_host, tuner_port);
            cmd::run_sse_converter(config, &tuner_url).await?;
            Ok(())
        }
        SubCommands::SseEpg {
            protocol,
            tuner_host,
            tuner_port,
        } => {
            let tuner_url = format!("{}://{}:{}", protocol, tuner_host, tuner_port);
            cmd::run_sse_epg(config, &tuner_url).await?;
            Ok(())
        }
        SubCommands::SseRecord {
            protocol,
            tuner_host,
            tuner_port,
        } => {
            let tuner_url = format!("{}://{}:{}", protocol, tuner_host, tuner_port);
            cmd::run_sse_record(config, &tuner_url).await?;
            Ok(())
        }
        SubCommands::RuleMeilisearch {} => {
            cmd::run_rule_meilisearch(config).await?;
            Ok(())
        }
        SubCommands::StreamRecord {} => {
            cmd::run_stream_record(config).await?;
            Ok(())
        }
    }
}
