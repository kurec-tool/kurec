use std::{error::Error, path::Path};

use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KurecConfig {
    pub tuners: Vec<String>,
    pub nats_host: String,
    pub meilisearch_host: String,
    pub meilisearch_api_key: Option<String>,
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
