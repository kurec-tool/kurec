use std::collections::HashMap;

use chrono::TimeZone;
use serde::{Deserialize, Serialize};

use crate::model::{common::get_subgenre, mirakurun};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramDocument {
    #[serde(rename = "programId")]
    pub id: u64,
    #[serde(rename = "チャンネル名")]
    pub service_name: String,
    #[serde(rename = "チャンネルタイプ")]
    pub channel_type: String,
    #[serde(rename = "開始時刻")]
    pub start_at: chrono::DateTime<chrono::Local>,
    #[serde(rename = "放送時間")]
    pub duration: u64,
    #[serde(rename = "タイトル")]
    pub title: String,
    #[serde(rename = "番組説明")]
    pub description: String,
    #[serde(rename = "拡張情報")]
    pub extended: HashMap<String, String>,
    #[serde(rename = "ジャンル")]
    pub genres: String,
}

impl ProgramDocument {
    pub fn get_searchable_attributes() -> Vec<&'static str> {
        vec!["タイトル", "番組説明", "拡張情報"]
    }
    pub fn get_filterable_attributes() -> Vec<&'static str> {
        vec!["チャンネル名", "ジャンル"]
    }
    pub fn get_sortable_attributes() -> Vec<&'static str> {
        vec!["開始時刻", "放送時間"]
    }

    pub fn from_mirakurun(
        program: &mirakurun::program::Program,
        service: &mirakurun::service::Service,
    ) -> Option<Self> {
        match chrono::Local.timestamp_millis_opt(program.start_at as i64) {
            chrono::LocalResult::None => None,
            chrono::LocalResult::Single(start_at) => Some(Self {
                id: program.id,
                service_name: service.name.clone(),
                channel_type: service.channel.channel_type.clone(),
                start_at,
                duration: program.duration / 1000,
                title: program.name.clone().unwrap_or_default(),
                description: program.description.clone().unwrap_or_default(),
                extended: program.extended.clone().unwrap_or_default(),
                genres: program
                    .genres
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    .map(|g| get_subgenre(g["lv1"], g["lv2"]).to_string())
                    .collect::<Vec<_>>()
                    .join("、"),
            }),
            chrono::LocalResult::Ambiguous(_, _) => None,
        }
    }
}
