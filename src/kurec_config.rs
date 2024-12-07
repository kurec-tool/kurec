use std::{error::Error, path::Path};

use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct KurecConfig {
    pub tuners: Vec<String>,
    pub nats: NatsConfig,
    pub minio: MinioConfig,
    pub meilisearch: MeilisearchConfig,

    #[serde(default = "default_prefix")]
    pub prefix: String,
}

fn default_prefix() -> String {
    "kurec".to_string()
}

#[derive(Clone, Debug, Deserialize)]
pub struct NatsConfig {
    pub url: String,
    pub kv: NatsKvConfig,
    pub stream: NatsStreamConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NatsKvConfig {
    #[serde(default = "default_kv_epg_bucket_config")]
    pub epg: NatsKvBucketConfig,
    #[serde(default = "default_kv_record_bucket_config")]
    pub record: NatsKvBucketConfig,
}

fn default_kv_epg_bucket_config() -> NatsKvBucketConfig {
    NatsKvBucketConfig {
        name: "epg".to_string(),
        max_age: 30 * 24 * 60 * 60,
        history: 10,
    }
}

fn default_kv_record_bucket_config() -> NatsKvBucketConfig {
    NatsKvBucketConfig {
        name: "record".to_string(),
        max_age: 30 * 24 * 60 * 60,
        history: 10,
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NatsKvBucketConfig {
    pub name: String,
    pub max_age: u64,
    pub history: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NatsStreamConfig {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MinioConfig {
    pub region: String,
    pub endpoint_url: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MeilisearchConfig {
    pub url: String,
}

pub fn get_config<P>(config_path: P) -> Result<KurecConfig, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    let config: KurecConfig = Figment::new()
        .merge(Yaml::file(config_path))
        .merge(Env::prefixed("KUREC"))
        .extract()?;

    Ok(config)
}
