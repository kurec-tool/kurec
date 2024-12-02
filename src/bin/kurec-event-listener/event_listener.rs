use actix::prelude::*;
use kurec::message::event::{parse_event, MirakcEventData};
use tracing::{debug, error, info};

#[derive(Message)]
#[rtype(result = "bool")]
pub struct Ping {}

pub(crate) struct EventListener {
    tuner_name: String,
    mirakc_resp: Option<reqwest::Response>,
    nats_client: async_nats::Client,
}

impl EventListener {
    pub fn new(
        tuner_name: String,
        mirakc_resp: reqwest::Response,
        nats_client: async_nats::Client,
    ) -> Self {
        EventListener {
            tuner_name,
            mirakc_resp: Some(mirakc_resp),
            nats_client,
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
                    let subject = match ev.data {
                        MirakcEventData::EpgProgramsUpdated { service_id } => format!(
                            "mirakc.event.{}.{}.{}",
                            ev.event, self.tuner_name, service_id
                        ),
                        _ => format!("mirakc.event.{}.{}", ev.event, self.tuner_name),
                    };
                    let nc = self.nats_client.clone();
                    async move {
                        match nc
                            .publish(subject.clone(), message_body.clone().into())
                            .await
                        {
                            Ok(_) => info!("published message[{}]: {}", subject, message_body),
                            Err(e) => error!("Failed to publish message: {:?}", e),
                        }
                    }
                    .into_actor(self)
                    .spawn(ctx);
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
