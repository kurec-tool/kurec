use std::collections::HashMap;

use derivative::Derivative;
use figment::{
    providers::{Env, Format, Serialized, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};

use crate::{StorageConfig, StorageType};

#[derive(Derivative, Debug, Clone, Deserialize, Serialize)]
#[derivative(Default)]
pub struct KurecConfig {
    #[derivative(Default(value = "\"kurec\".to_string()"))]
    pub prefix: String,

    #[derivative(Default(value = "300"))]
    pub ogp_width: u32,

    #[derivative(Default(value = "true"))]
    pub color_log: bool,

    #[derivative(Default(value = "false"))]
    pub json_log: bool,

    pub tuners: HashMap<String, String>,

    pub nats: NatsConfig,

    pub meilisearch: MeilisearchConfig,

    pub encoder: EncoderConfig,

    pub storage: StorageConfig,
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

// TODO: KurecConfigじゃなくてkurec-interface/src/document/program.rsに定義を移動する
impl Default for MeilisearchConfig {
    fn default() -> Self {
        Self {
            url: "http://meilisearch:7700".to_string(),
            api_key: None,
            epg: MeilisearchIndexConfig {
                index_base_name: "epg".to_string(),
                primary_key: "program_id".to_string(),
                filterable_attributes: vec![
                    "ジャンル".to_string(),
                    "放送局".to_string(),
                    "放送曜日".to_string(),
                ],
                searchable_attributes: vec![
                    "タイトル".to_string(),
                    "番組情報".to_string(),
                    "その他情報".to_string(),
                ],
                displayed_attributes: vec![
                    "program_id".to_string(),
                    "タイトル".to_string(),
                    "番組情報".to_string(),
                    "その他情報".to_string(),
                    "開始時刻".to_string(),
                    "終了時刻".to_string(),
                    "放送曜日".to_string(),
                    "放送局".to_string(),
                    "ジャンル".to_string(),
                    "放送時間".to_string(),
                    "公式サイト等".to_string(),
                    "ogp_url_hash".to_string(),
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EncoderOutput {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub storage: StorageType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EncoderConfig {
    pub script: String,
    pub outputs: Vec<EncoderOutput>,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            script: "ffmpeg -i input.ts -c:a copy output.mp4".to_string(),
            outputs: vec![
                EncoderOutput {
                    name: "output.mp4".to_string(),
                    description: "MP4(H.264)".to_string(),
                    type_: "video/mp4".to_string(),
                    storage: StorageType::Local,
                },
                EncoderOutput {
                    name: "input.ts".to_string(),
                    description: "元TS".to_string(),
                    type_: "video/MP2T".to_string(),
                    storage: StorageType::Local,
                },
                EncoderOutput {
                    name: "metadata.json".to_string(),
                    description: "メタデータ".to_string(),
                    type_: "application/json".to_string(),
                    storage: StorageType::Local,
                },
            ],
        }
    }
}
