use std::collections::HashMap;

use derivative::Derivative;
use figment::{
    providers::{Env, Format, Serialized, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};

#[derive(Derivative, Debug, Clone, Deserialize, Serialize)]
#[derivative(Default)]
pub struct KurecConfig {
    #[derivative(Default(value = "\"kurec\".to_string()"))]
    pub prefix: String,

    #[derivative(Default(value = "true"))]
    pub color_log: bool,

    #[derivative(Default(value = "false"))]
    pub json_log: bool,

    pub tuners: HashMap<String, String>,

    pub nats: NatsConfig,

    pub meilisearch: MeilisearchConfig,
}

impl KurecConfig {
    pub fn get_config() -> Result<Self, figment::Error> {
        let figment = Figment::new()
            .merge(Serialized::defaults(KurecConfig::default()))
            .merge(Env::prefixed("KUREC_"))
            .merge(Yaml::file("kurec.yml"));

        figment.extract()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct NatsConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MeilisearchConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub epg: MeilisearchIndexConfig,
}

impl Default for MeilisearchConfig {
    fn default() -> Self {
        Self {
            url: "http://meilisearch:7700".to_string(),
            api_key: None,
            epg: MeilisearchIndexConfig {
                index_base_name: "epg".to_string(),
                primary_key: "program_id".to_string(),
                filterable_attributes: vec!["ジャンル".to_string(), "放送局".to_string()],
                searchable_attributes: vec![
                    "タイトル".to_string(),
                    "番組情報".to_string(),
                    "その他情報".to_string(),
                ],
                displayed_attributes: vec![
                    "タイトル".to_string(),
                    "番組情報".to_string(),
                    "その他情報".to_string(),
                    "開始時刻".to_string(),
                    "終了時刻".to_string(),
                    "放送局".to_string(),
                    "ジャンル".to_string(),
                    "放送時間".to_string(),
                    "公式サイト等".to_string(),
                ],
                sortable_attributes: vec!["開始時刻".to_string()],
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MeilisearchIndexConfig {
    pub index_base_name: String,
    pub primary_key: String,
    pub filterable_attributes: Vec<String>,
    pub searchable_attributes: Vec<String>,
    pub displayed_attributes: Vec<String>,
    pub sortable_attributes: Vec<String>,
}
