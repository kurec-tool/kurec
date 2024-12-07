use anyhow::Result;
use async_nats::jetstream::consumer::PullConsumer;
use futures::StreamExt;
use kurec::{
    config::KurecConfig,
    model::{meilisearch::MeilisearchQuery, mirakurun::program::Program},
};
use std::fs::File;
use std::io::BufReader;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

pub async fn run_rule_meilisearch(config: KurecConfig) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let client = async_nats::connect(&config.nats_url).await?;
    let jetstream = async_nats::jetstream::new(client);

    let stream_name = config.get_epg_stream_name();

    // TODO: ファイルからじゃなくKVから読む
    let query1 = {
        let file = File::open("testdata/meilisearch/query1.json")?;
        let reader = BufReader::new(file);
        let query1: MeilisearchQuery = serde_json::from_reader(reader)?;
        query1
    };
    let query2 = {
        let file = File::open("testdata/meilisearch/query2.json")?;
        let reader = BufReader::new(file);
        let query2: MeilisearchQuery = serde_json::from_reader(reader)?;
        query2
    };
    let queries = vec![query1.clone(), query2.clone()];

    let stream = jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: stream_name.to_string(),
            max_messages: 10_000_000,
            ..Default::default()
        })
        .await?;

    let consumer: PullConsumer = stream
        .get_or_create_consumer(
            "kurec-epg-rule-meilisearch", // この名前の意味は？
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some("kurec-epg-rule-meilisearch".to_string()),
                // TODO: Config調整
                ..Default::default()
            },
        )
        .await?;

    let mut messages = consumer.messages().await?;
    loop {
        match messages.next().await {
            Some(Ok(msg)) => {
                let message: kurec::message::jetstream_message::OnEpgProgramUpdated =
                    match serde_json::from_slice(&msg.payload) {
                        Ok(m) => m,
                        Err(e) => {
                            error!("Error when deserializing message: {:?}", e);
                            continue;
                        }
                    };
                let tuner_url = message.tuner_url.clone();
                let program_ids = message.program_ids;
                info!(
                    "received message tuner_url:[{}] program_ids count:{}",
                    tuner_url,
                    program_ids.len()
                );

                let scheduled_program_ids =
                    match kurec::adapter::mirakc::list_scheduled_program_ids(&tuner_url).await {
                        Ok(ids) => ids,
                        Err(e) => {
                            error!("Error when listing scheduled program ids: {:?}", e);
                            continue;
                        }
                    };

                debug!("start getting programs...");
                let programs: Vec<Program> = kurec::adapter::mirakc::list_programs_by_service_id(
                    &tuner_url,
                    message.service_id,
                )
                .await?
                .iter()
                .filter(|p| program_ids.contains(&p.id))
                .cloned()
                .collect();

                debug!("done.");

                let service =
                    match kurec::adapter::mirakc::get_service(&tuner_url, message.service_id).await
                    {
                        Ok(s) => s,
                        Err(e) => {
                            error!("Error when getting service: {:?}", e);
                            continue;
                        }
                    };

                let hits = match kurec::adapter::meilisearch::list_rule_matched_program_ids(
                    &config.meilisearch_url,
                    &config.meilisearch_api_key,
                    &service,
                    &programs,
                    &query1,
                )
                .await
                {
                    Ok(hits) => hits,
                    Err(e) => {
                        error!("Error when searching program: {:?}", e);
                        continue;
                    }
                };
                debug!("hits count: {}", hits.len());
                for program_id in program_ids.iter() {
                    let is_hit = hits.contains(program_id);
                    // debug!("program_id: {} is_hit: {}", program_id, is_hit);

                    if is_hit {
                        if scheduled_program_ids.contains(program_id) {
                            continue;
                        }
                        info!("scheduling program_id: {}", program_id);
                        match kurec::adapter::mirakc::schedule_program(&tuner_url, *program_id)
                            .await
                        {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Error when scheduling program: {:?}", e);
                                continue;
                            }
                        };
                    } else {
                        if !scheduled_program_ids.contains(program_id) {
                            continue;
                        }
                        info!("unscheduling program_id: {}", program_id);
                        match kurec::adapter::mirakc::unschedule_program(&tuner_url, *program_id)
                            .await
                        {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Error when unscheduling program: {:?}", e);
                                continue;
                            }
                        };
                    }
                }

                msg.ack().await.unwrap();
            }
            Some(Err(e)) => {
                error!("Error when reading messages: {:?}", e);
            }
            None => {
                error!("No more messages");
                break;
            }
        }
    }

    error!("End of the program");

    Ok(())
}
