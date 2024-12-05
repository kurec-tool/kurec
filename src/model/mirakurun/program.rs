// Example code that deserializes and serializes the model.
// extern crate serde;
// #[macro_use]
// extern crate serde_derive;
// extern crate serde_json;
//
// use generated_module::Programs;
//
// fn main() {
//     let json = r#"{"answer": 42}"#;
//     let model: Programs = serde_json::from_str(&json).unwrap();
// }

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Programs = Vec<Program>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Program {
    pub id: u64,
    pub event_id: u64,
    pub service_id: u64,
    pub network_id: u64,
    pub start_at: u64,
    pub duration: u64,
    pub is_free: bool,
    pub name: Option<String>,
    pub description: Option<String>,
    pub extended: Option<HashMap<String, String>>,
    pub video: Option<Video>,
    pub audio: Option<Audio>,
    pub audios: Option<Vec<Audio>>,
    pub genres: Option<Vec<HashMap<String, u8>>>,
    pub related_items: Option<Vec<RelatedItem>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audio {
    pub component_type: u64,
    pub is_main: bool,
    pub sampling_rate: u64,
    pub langs: Vec<Lang>,
}

pub type Lang = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedItem {
    #[serde(rename = "type")]
    pub related_item_type: RelatedItemType,
    pub network_id: Option<serde_json::Value>,
    pub service_id: u64,
    pub event_id: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelatedItemType {
    Shared,
    Relay,
    Movement,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    #[serde(rename = "type")]
    pub video_type: VideoType,
    pub resolution: Resolution,
    pub stream_content: u64,
    pub component_type: u64,
}

pub type Resolution = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoType {
    Mpeg2,
}
