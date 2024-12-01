use actix::{Actor, Addr};
use actix_web::{get, web, App, HttpResponse, HttpServer};
use event_listener::EventListener;
use tracing::error;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::EnvFilter;

mod event_listener;

#[derive(Clone)]
struct AppData {
    event_listener: Addr<EventListener>,
}

impl AppData {
    fn new(event_listener: Addr<EventListener>) -> Self {
        Self { event_listener }
    }
}

#[get("/health")]
async fn health(data: web::Data<AppData>) -> HttpResponse {
    match data.event_listener.send(event_listener::Ping {}).await {
        Ok(true) => HttpResponse::Ok().finish(),
        _ => HttpResponse::ServiceUnavailable().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = kurec::kurec_config::get_config("./kurec.yml").unwrap();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let mut mirakc_url = config.mirakc.url.clone();
    if mirakc_url.ends_with('/') {
        mirakc_url.pop();
    }
    let events_url = format!("{}/events", mirakc_url);

    let mirakc_resp = match reqwest::get(events_url).await {
        Ok(resp) => resp,
        Err(e) => {
            error!("mirakc connect error: {:?}", e);
            std::process::exit(1);
        }
    };

    let nc = match nats::connect(config.nats.host) {
        Ok(nc) => nc,
        Err(e) => {
            error!("nats connect error: {:?}", e);
            std::process::exit(1);
        }
    };
    let event_listener = EventListener::new(mirakc_resp, nc).start();
    let data = AppData::new(event_listener);

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(health)
            .app_data(web::Data::new(data.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
