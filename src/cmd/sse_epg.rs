use anyhow::Result;
use bytes::Bytes;
use futures::StreamExt;
use kurec::adapter::{mirakc, sse_stream::get_sse_service_id_stream};
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

use kurec::config::KurecConfig;

pub async fn run_sse_epg(config: KurecConfig, tuner_url: &str) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let bucket_name = config.get_epg_bucket_name();
    let stream_name = config.get_epg_stream_name();
    let nats_url = config.nats_url;
    let client = async_nats::connect(nats_url).await?;
    let jetstream = async_nats::jetstream::new(client);
    let kv = match jetstream.get_key_value(bucket_name.to_string()).await {
        Ok(kv) => kv,
        Err(e) => {
            error!("Error: {:?}", e);
            jetstream
                .create_key_value(async_nats::jetstream::kv::Config {
                    bucket: bucket_name.to_string(),
                    max_age: config.epg_kv_max_age.into(), // 30 days
                    history: config.epg_kv_history,        // 適当
                    // TODO: パラメータ調整
                    ..Default::default()
                })
                .await?
        }
    };
    // consumer作るときにはstreamいるけど、publishだけならいらないが、
    // Config設定する必要があるのでget_or_create_streamを使う
    let _ = jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: stream_name.to_string(),
            max_messages: 10_000_000,
            // TODO: Config調整
            ..Default::default()
        })
        .await?;

    match get_sse_service_id_stream(tuner_url).await {
        Ok(mut stream) => {
            while let Some(service_id) = stream.next().await {
                let programs = match mirakc::get_json_programs(tuner_url, service_id).await {
                    Ok(programs) => {
                        // debug!("got programs: {:?}", programs.len());
                        programs
                    }
                    Err(e) => {
                        error!("Error: {:?}", e);
                        continue;
                    }
                };

                let mut program_ids: Vec<u64> = Vec::with_capacity(programs.len());

                for program in &programs {
                    let program_id_str = program.id.to_string();
                    let json_bytes: Bytes = program.json.clone().into();
                    let kv_bytes = match kv.get(program_id_str.clone()).await {
                        Ok(kv_bytes) => kv_bytes,
                        Err(e) => {
                            error!("Error: {:?}", e);
                            continue;
                        }
                    };
                    if kv_bytes.is_none() || kv_bytes.unwrap() != json_bytes {
                        // TODO: updateにしてちゃんとリビジョン処理したり、available_tunersを更新したりする
                        match kv.put(program_id_str.clone(), json_bytes).await {
                            Ok(_) => {
                                debug!("program_id: {} recorded", program.id);
                            }
                            Err(e) => {
                                error!("Error: {:?}", e);
                                continue;
                            }
                        };
                        program_ids.push(program.id);
                    }
                }
                let message = kurec::message::jetstream_message::OnEpgProgramUpdated {
                    tuner_url: tuner_url.to_string(),
                    service_id,
                    program_ids: program_ids.clone(),
                };
                let message_vec = match serde_json::to_vec(&message) {
                    Ok(message_vec) => message_vec,
                    Err(e) => {
                        error!("JSON serialize error: {:?}", e);
                        continue;
                    }
                };
                match jetstream
                    .publish(stream_name.clone(), message_vec.into())
                    .await
                {
                    Ok(_) => {
                        info!("program_ids.len:{:?} published", program_ids.len());
                    }
                    Err(e) => {
                        error!("Error: {:?}", e);
                        continue;
                    }
                }
            }
            error!("mirakc events stream ended");

            Err(anyhow::anyhow!("mirakc events stream ended"))
        }
        Err(e) => {
            error!("Error: {:?}", e);
            Err(e)
        }
    }
}
