use actix::prelude::*;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use actor::{event_listener::EventListener, program_register::ProgramRegister};
use kurec::message::ping_pong::Ping;
use tracing::error;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::EnvFilter;

mod actor;

#[derive(Clone)]
struct AppData {
    event_listeners: Vec<Addr<EventListener>>,
    program_register: Addr<ProgramRegister>,
}

#[get("/health")]
async fn health(data: web::Data<AppData>) -> HttpResponse {
    for event_listener in &data.event_listeners {
        match event_listener.send(Ping {}).await {
            Ok(true) => continue,
            _ => {
                error!("event_listener is not available");
                return HttpResponse::ServiceUnavailable().finish();
            }
        }
    }
    match data.program_register.send(Ping {}).await {
        Ok(true) => {}
        _ => {
            error!("program_register is not available");
            return HttpResponse::ServiceUnavailable().finish();
        }
    }

    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("TZ", "Asia/Tokyo");
    let config = kurec::kurec_config::get_config("./kurec.yml").unwrap();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .init();

    let config_clone = config.clone();
    let arbiter = Arbiter::current();
    let program_register = Supervisor::start_in_arbiter(&arbiter, move |_| {
        ProgramRegister::new(
            config_clone.tuners[0].clone(),
            config_clone.meilisearch_host.clone(),
            config_clone.meilisearch_api_key.clone(),
        )
    });
    let mut event_listeners = Vec::new();

    for tuner in &config.tuners {
        let tuner = tuner.clone();
        let program_register = program_register.clone();
        let event_listener = Supervisor::start_in_arbiter(&arbiter, move |_| {
            EventListener::new(tuner, program_register)
        });
        event_listeners.push(event_listener);
    }

    let data = AppData {
        event_listeners,
        program_register,
    };

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
