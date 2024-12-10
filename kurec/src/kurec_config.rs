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

#[derive(Debug, Clone, Deserialize)]
pub struct MeilisearchConfig {
    pub url: String,
}
