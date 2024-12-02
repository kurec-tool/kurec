use actix::{Actor, Addr};
use actix_web::{get, web, App, HttpResponse, HttpServer};
use event_listener::EventListener;
use tracing::error;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::EnvFilter;

mod event_listener;

#[derive(Clone)]
struct AppData {
    event_listeners: Vec<Addr<EventListener>>,
}

impl AppData {
    fn new(event_listeners: Vec<Addr<EventListener>>) -> Self {
        Self { event_listeners }
    }
}

#[get("/health")]
async fn health(data: web::Data<AppData>) -> HttpResponse {
    for event_listener in &data.event_listeners {
        match event_listener.send(event_listener::Ping {}).await {
            Ok(true) => continue,
            _ => {
                error!("event_listener is not available");
                return HttpResponse::ServiceUnavailable().finish();
            }
        }
    }
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = kurec::kurec_config::get_config("./kurec.yml").unwrap();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let mut event_listeners = Vec::new();

    for (tuner_name, tuner_url) in &config.mirakc.tuners {
        let mut tuner_url = tuner_url.clone();
        if tuner_url.ends_with('/') {
            tuner_url.pop();
        }
        let event_url = format!("{}/events", tuner_url);
        let mirakc_resp = match reqwest::get(event_url).await {
            Ok(resp) => resp,
            Err(e) => {
                error!("mirakc connect error: {:?}", e);
                std::process::exit(1);
            }
        };

        let nc = match async_nats::connect(config.nats.host.clone()).await {
            Ok(nc) => nc,
            Err(e) => {
                error!("nats connect error: {:?}", e);
                std::process::exit(1);
            }
        };
        let event_listener = EventListener::new(tuner_name.clone(), mirakc_resp, nc).start();
        event_listeners.push(event_listener);
    }

    let data = AppData::new(event_listeners);

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
