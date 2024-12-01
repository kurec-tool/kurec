use std::{error::Error, path::Path};

use figment::{
    providers::{Env, Format, Serialized, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct MirakcConfig {
    #[validate(url)]
    pub url: String,
}

impl Default for MirakcConfig {
    fn default() -> Self {
        Self {
            url: "http://mirakc:40772/".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct NatsConfig {
    pub host: String,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            host: "nats".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Validate)]
pub struct KurecConfig {
    pub mirakc: MirakcConfig,
    pub nats: NatsConfig,
}

pub fn get_config<P>(config_path: P) -> Result<KurecConfig, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    let config: KurecConfig = Figment::from(Serialized::defaults(KurecConfig::default()))
        .merge(Yaml::file(config_path))
        .merge(Env::prefixed("KUREC"))
        .extract()?;
    config.validate()?;

    Ok(config)
}
