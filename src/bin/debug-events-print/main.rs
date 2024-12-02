use futures::StreamExt;
use kurec::message::event::MirakcEvent;
use tracing::error;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = kurec::kurec_config::get_config("./kurec.yml").unwrap();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let nc = match async_nats::connect(config.nats.host).await {
        Ok(nc) => nc,
        Err(e) => {
            error!("nats connect error: {:?}", e);
            std::process::exit(1);
        }
    };
    let mut sub = nc.subscribe("mirakc.event.>").await.unwrap();

    while let Some(msg) = sub.next().await {
        println!(
            "{:?} {:?}",
            msg,
            serde_json::from_slice::<MirakcEvent>(msg.payload.as_ref())
        );
    }

    Ok(())
}
