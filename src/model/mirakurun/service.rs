// Example code that deserializes and serializes the model.
// extern crate serde;
// #[macro_use]
// extern crate serde_derive;
// extern crate serde_json;
//
// use generated_module::Services;
//
// fn main() {
//     let json = r#"{"answer": 42}"#;
//     let model: Services = serde_json::from_str(&json).unwrap();
// }

use serde::{Deserialize, Serialize};

pub type Services = Vec<Service>;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: u64,
    pub service_id: u64,
    pub network_id: u64,
    #[serde(rename = "type")]
    pub service_type: u64,
    pub logo_id: u64,
    pub remote_control_key_id: u64,
    pub name: String,
    pub channel: Channel,
    pub has_logo_data: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Channel {
    #[serde(rename = "type")]
    pub channel_type: Type,
    pub channel: String,
}

pub type Type = String;
