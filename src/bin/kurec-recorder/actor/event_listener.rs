use actix::prelude::*;
use kurec::message::{event::parse_event, ping_pong::Ping};
use tracing::{debug, error, info, warn};

use crate::actor::program_register;

use super::program_register::ProgramRegister;

pub(crate) struct EventListener {
    tuner: String,
    program_register: Addr<ProgramRegister>,
}

impl EventListener {
    pub fn new(tuner: String, program_register: Addr<ProgramRegister>) -> Self {
        EventListener {
            tuner,
            program_register,
        }
    }
}

impl Actor for EventListener {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("EventListener started");
        let events_url = format!("http://{}:40772/events", self.tuner);
        async move {
            match reqwest::get(events_url).await {
                Ok(resp) => Ok(resp),
                Err(e) => {
                    error!("mirakc connect error: {:?}", e);
                    Err(e)
                }
            }
        }
        .into_actor(self)
        .map(|res, _act, ctx| match res {
            Ok(resp) => {
                ctx.add_stream(resp.bytes_stream());
            }
            Err(e) => {
                error!("mirakc connect error: {:?}", e);
                ctx.stop();
            }
        })
        .wait(ctx);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        warn!("EventListener stopped");
    }
}

impl Supervised for EventListener {
    fn restarting(&mut self, _ctx: &mut Self::Context) {
        warn!("EventListener restarting");
    }
}

impl StreamHandler<Result<bytes::Bytes, reqwest::Error>> for EventListener {
    fn handle(&mut self, item: Result<bytes::Bytes, reqwest::Error>, ctx: &mut Self::Context) {
        match item {
            Ok(s) => {
                let ev = parse_event(s.as_ref());
                if let Some(ev) = ev {
                    debug!("event: {:?}", ev);
                    match ev.data {
                        kurec::message::event::MirakcEventData::EpgProgramsUpdated {
                            service_id,
                        } => {
                            debug!("epg program updated: {}", service_id);
                            self.program_register
                                .do_send(program_register::UpdateEpgProgramsMessage { service_id });
                        }
                        _ => {
                            debug!("ignore event");
                        }
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
