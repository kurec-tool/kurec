use std::{collections::HashMap, error::Error, path::Path};

use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};
use url::Url;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct MirakcConfig {
    #[validate(custom(function = "validate_tuners"))]
    pub tuners: HashMap<String, String>,
}

fn validate_tuners(tuners: &HashMap<String, String>) -> Result<(), ValidationError> {
    if tuners.is_empty() {
        return Err(ValidationError::new("tuners is empty"));
    }
    for value in tuners.values() {
        if Url::parse(value).is_err() {
            return Err(ValidationError::new("tuner url parse error"));
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct NatsConfig {
    pub host: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct KurecConfig {
    pub mirakc: MirakcConfig,
    pub nats: NatsConfig,
}

pub fn get_config<P>(config_path: P) -> Result<KurecConfig, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    let config: KurecConfig = Figment::new()
        .merge(Yaml::file(config_path))
        .merge(Env::prefixed("KUREC"))
        .extract()?;
    config.validate()?;

    Ok(config)
}
