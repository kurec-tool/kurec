use actix::prelude::*;
use kurec::message::event::parse_event;
use tracing::{debug, error, info};

#[derive(Message)]
#[rtype(result = "bool")]
pub struct Ping {}

pub(crate) struct EventListener {
    mirakc_resp: Option<reqwest::Response>,
    nats_connection: nats::Connection,
}

impl EventListener {
    pub fn new(mirakc_resp: reqwest::Response, nats_connection: nats::Connection) -> Self {
        EventListener {
            mirakc_resp: Some(mirakc_resp),
            nats_connection,
        }
    }
}

impl Actor for EventListener {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("EventListener started");
        if let Some(resp) = self.mirakc_resp.take() {
            ctx.add_stream(resp.bytes_stream());
        } else {
            error!("can't start EventListener");
            ctx.stop();
        }
    }
}

impl StreamHandler<Result<bytes::Bytes, reqwest::Error>> for EventListener {
    fn handle(&mut self, item: Result<bytes::Bytes, reqwest::Error>, ctx: &mut Self::Context) {
        match item {
            Ok(s) => {
                let ev = parse_event(s.as_ref());
                if let Some(ev) = ev {
                    debug!("event: {:?}", ev);
                    let message_body = match serde_json::to_string(&ev) {
                        Ok(body) => body,
                        Err(e) => {
                            error!("json parse error: {:?}", e);
                            return;
                        }
                    };
                    if let Err(e) = self
                        .nats_connection
                        .publish("mirakc.event", message_body.as_bytes())
                    {
                        error!("Failed to publish message: {:?}", e);
                    }
                }
            }
            Err(e) => {
                error!("sse message decode error: {:?}", e);
                ctx.stop();
            }
        }
    }
}

impl Handler<Ping> for EventListener {
    type Result = bool;
    fn handle(&mut self, _msg: Ping, _ctx: &mut Self::Context) -> bool {
        true
    }
}
