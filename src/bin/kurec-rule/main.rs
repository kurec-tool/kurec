use anyhow::Result;
use futures::StreamExt;
use mirakc::get_programs;
use rule::apply_rule;
use sse_stream::get_sse_service_id_stream;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

mod meili_rule;
mod meilisearch;
mod mirakc;
mod rule;
mod sse_stream;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let tuner_url = "http://tuner:40772";

    match get_sse_service_id_stream(tuner_url).await {
        Ok(mut stream) => {
            while let Some(service_id) = stream.next().await {
                debug!("service_id: {}", service_id);
                let service = match mirakc::get_service(tuner_url, service_id).await {
                    Ok(service) => service,
                    Err(e) => {
                        error!("Error: {:?}", e);
                        continue;
                    }
                };

                let programs = match get_programs(tuner_url, service_id).await {
                    Ok(programs) => {
                        debug!("got programs: {:?}", programs.len());
                        programs
                    }
                    Err(e) => {
                        error!("Error: {:?}", e);
                        continue;
                    }
                };

                let num_applied = match apply_rule(&programs, &service).await {
                    Ok(num_applied) => {
                        debug!("num_applied: {}", num_applied);
                        num_applied
                    }
                    Err(e) => {
                        error!("Error: {:?}", e);
                        continue;
                    }
                };
                info!(
                    "service_id: {} apply rule done. {} programs will be recorded.",
                    service_id, num_applied
                );
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
