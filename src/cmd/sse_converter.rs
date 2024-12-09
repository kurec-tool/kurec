use std::{str::from_utf8, time::Duration};

use anyhow::Result;
use futures::StreamExt;
use kurec::adapter::sse_stream::get_sse_stream;
use serde_json::json;
use tracing::{debug, error};
use tracing_subscriber::EnvFilter;

use kurec::config::KurecConfig;

const EVENT_HEADER: &[u8] = b"event: ";
const DATA_HEADER: &[u8] = b"data: ";

pub async fn run_sse_converter(config: KurecConfig, tuner_url: &str) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let nats_url = config.nats_url;
    let client = async_nats::connect(nats_url).await?;
    let jetstream = async_nats::jetstream::new(client);

    match get_sse_stream(tuner_url).await {
        Ok(mut stream) => {
            debug!("connect to {tuner_url} success.");
            while let Some(payload) = stream.next().await {
                let payload = match payload {
                    Ok(payload) => payload,
                    Err(e) => {
                        error!("SSE read error: {:?}", e);
                        continue;
                    }
                };
                if !payload.starts_with(EVENT_HEADER) {
                    // event: で始まらないデータは "\n\n\n" とかなので無視
                    continue;
                }
                // "event: " の後、改行までの部分を取り出す
                let (event, data) = match payload.iter().position(|&b| b == b'\n') {
                    Some(pos) => {
                        // event はここまで
                        let event = match from_utf8(&payload[EVENT_HEADER.len()..pos]) {
                            Ok(event) => event,
                            Err(e) => {
                                error!("SSE event header decode error: {:?}", e);
                                continue;
                            }
                        };
                        // その次の行は "data: " で始まるので、改行までの部分を取り出す
                        let data_start = pos + 1;
                        if !payload[data_start..].starts_with(DATA_HEADER) {
                            error!("SSE data header not found");
                            continue;
                        }
                        let data_start = data_start + DATA_HEADER.len();
                        let data_end = match payload[data_start..].iter().position(|&b| b == b'\n')
                        {
                            Some(pos) => data_start + pos,
                            None => {
                                error!("SSE data header not terminated by newline");
                                continue;
                            }
                        };
                        let data = match from_utf8(&payload[data_start..data_end]) {
                            Ok(data) => data,
                            Err(e) => {
                                error!("SSE data decode error: {:?}", e);
                                continue;
                            }
                        };
                        (event, data)
                    }
                    None => {
                        error!("SSE event header not found");
                        continue;
                    }
                };
                let subject = format!("{}-sse-{}", config.prefix, event).replace(".", "-");
                let payload = json!({
                    "event": event,
                    "data": data,
                    "tuner": tuner_url,
                });

                // TODO: 設定テーブルを用意してイベントごとに設定を変えられるようにする
                // 特にmax_ageは重要
                let _ = jetstream
                    .get_or_create_stream(async_nats::jetstream::stream::Config {
                        name: subject.clone(),
                        max_messages: 10_000_000,
                        max_age: Duration::from_secs(7 * 24 * 60 * 60), // これどうするかなー
                        // TODO: Config調整
                        ..Default::default()
                    })
                    .await?;

                match jetstream
                    .publish(subject.clone(), payload.to_string().into())
                    .await
                {
                    Ok(ret) => {
                        match ret.await {
                            Ok(ret) => {
                                debug!("published to {subject} with sequence: {:?}", ret);
                            }
                            Err(e) => {
                                error!("publish to {subject} error: {:?}", e);
                            }
                        };
                    }
                    Err(e) => {
                        error!("publish to {subject} error: {:?}", e);
                    }
                };
            }
            error!("mirakc events stream ended");

            Err(anyhow::anyhow!("mirakc events stream ended"))
        }
        Err(e) => {
            error!("connect to {tuner_url} error: {:?}", e);
            Err(e)
        }
    }
}
