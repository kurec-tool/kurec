use clap::{Args, Parser, Subcommand};
use kurec_adapter::{
    MeilisearchAdapter, MirakcAdapter, MirakcEventsAdapter, NatsAdapter, OgpAdapter,
};
use tracing_subscriber::EnvFilter;

use kurec_interface::KurecConfig;

mod domain;

#[derive(Clone, Debug, Parser)]
#[clap(name = env!("CARGO_PKG_NAME"), version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = env!("CARGO_PKG_DESCRIPTION"))]
struct Cli {
    #[clap(subcommand)]
    subcommand: SubCommands,
}

#[derive(Clone, Debug, Subcommand)]
enum SubCommands {
    Events {
        #[clap(index = 1)]
        tuner_name: String,
    },
    Epg(EpgArgs),
    Ogp {},
}

#[derive(Clone, Debug, Args)]
struct EpgArgs {
    #[clap(subcommand)]
    subcommand: EpgSubCommands,
}

#[derive(Clone, Debug, Subcommand)]
enum EpgSubCommands {
    Collector,
    Converter,
    Indexer,
    Filter,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = KurecConfig::get_config()?;

    let subscriber = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env());

    if config.json_log {
        subscriber.json().init();
    } else {
        subscriber.with_ansi(config.color_log).init();
    }

    match cli.subcommand {
        SubCommands::Events { tuner_name } => {
            println!("Tuner name: {}", tuner_name);
            let mirakc_adapter = MirakcEventsAdapter::try_new(config.clone(), &tuner_name).await?;
            let nats_adapter = NatsAdapter::new(config.clone());
            let events_domain = domain::EventsDomain::new(mirakc_adapter, nats_adapter);
            events_domain.copy_events_to_jetstream().await?;
            Ok(())
        }
        SubCommands::Epg(epg_args) => match epg_args.subcommand {
            EpgSubCommands::Collector => {
                let mirakc_adapter = MirakcAdapter::new(config.clone());
                let nats_adapter = NatsAdapter::new(config.clone());
                let meilisearch_adapter = MeilisearchAdapter::new_async(config.clone()).await?;
                let epg_domain =
                    domain::EpgDomain::new(mirakc_adapter, nats_adapter, meilisearch_adapter);
                epg_domain.collect_epg_stream().await?;
                Ok(())
            }
            EpgSubCommands::Converter => {
                let mirakc_adapter = MirakcAdapter::new(config.clone());
                let nats_adapter = NatsAdapter::new(config.clone());
                let meilisearch_adapter = MeilisearchAdapter::new_async(config.clone()).await?;
                let epg_domain =
                    domain::EpgDomain::new(mirakc_adapter, nats_adapter, meilisearch_adapter);
                epg_domain.convert_epg_stream().await?;
                Ok(())
            }
            EpgSubCommands::Indexer => {
                let mirakc_adapter = MirakcAdapter::new(config.clone());
                let nats_adapter = NatsAdapter::new(config.clone());
                let meilisearch_adapter = MeilisearchAdapter::new_async(config.clone()).await?;
                let epg_domain =
                    domain::EpgDomain::new(mirakc_adapter, nats_adapter, meilisearch_adapter);
                epg_domain.index_epg_stream().await?;
                Ok(())
            }
            EpgSubCommands::Filter => {
                println!("Filter");
                Ok(())
            }
        },
        SubCommands::Ogp {} => {
            let nats_adapter = NatsAdapter::new(config.clone());
            let ogp_adapter = OgpAdapter::new(config.clone());
            let ogp_domain = domain::OgpDomain::new(config.clone(), nats_adapter, ogp_adapter);
            ogp_domain.collect_ogp_stream().await?;
            Ok(())
        }
    }
}
