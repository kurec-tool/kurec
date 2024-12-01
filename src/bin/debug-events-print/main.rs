use kurec::message::event::MirakcEvent;
use tracing::error;
use tracing_subscriber::EnvFilter;

fn main() -> std::io::Result<()> {
    let config = kurec::kurec_config::get_config("./kurec.yml").unwrap();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let nc = match nats::connect(config.nats.host) {
        Ok(nc) => nc,
        Err(e) => {
            error!("nats connect error: {:?}", e);
            std::process::exit(1);
        }
    };
    let sub = nc.subscribe("mirakc.event")?;

    while let Some(msg) = sub.next() {
        println!(
            "{:?} {:?}",
            msg,
            serde_json::from_slice::<MirakcEvent>(msg.data.as_ref())
        );
    }

    Ok(())
}
