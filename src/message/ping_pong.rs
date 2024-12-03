use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "bool")]
pub struct Ping {}
