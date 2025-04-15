use derivative::Derivative;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageType {
    Local,
    S3,
}

#[derive(Clone, Derivative, Debug, Deserialize, Serialize)]
#[derivative(Default)]
#[serde(rename_all = "snake_case")]
pub struct StorageConfig {
    pub local_path: String,
}
