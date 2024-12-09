use std::time::Duration;

use duration_string::DurationString;
use envconfig::Envconfig;

#[derive(Clone, Debug, Envconfig)]
pub struct KurecConfig {
    /// Prefix for NATS KV bucket name, stream name, etc.
    #[envconfig(from = "KUREC_PREFIX", default = "kurec")]
    pub prefix: String,

    #[envconfig(from = "NATS_URL")]
    pub nats_url: String,

    #[envconfig(from = "NATS_EPG_KV_BUCKET_NAME", default = "epg")]
    epg_kv_bucket_name: String,

    #[envconfig(from = "NATS_EPG_KV_MAX_AGE", default = "30d")]
    pub epg_kv_max_age: MyDuration,

    #[envconfig(from = "NATS_EPG_KV_HISTORY", default = "10")]
    pub epg_kv_history: i64,

    #[envconfig(from = "NATS_RECORD_KV_BUCKET_NAME", default = "record")]
    record_kv_bucket_name: String,

    #[envconfig(from = "NATS_RECORD_KV_MAX_AGE", default = "30d")]
    pub record_kv_max_age: MyDuration,

    #[envconfig(from = "NATS_RECORD_KV_HISTORY", default = "10")]
    pub record_kv_history: i64,

    #[envconfig(from = "NATS_EPG_STREAM_NAME", default = "epg")]
    epg_stream_name: String,

    #[envconfig(from = "NATS_RECORD_STREAM_NAME", default = "record")]
    record_stream_name: String,

    #[envconfig(from = "NATS_RECORD_OBJECT_STORE_NAME", default = "record")]
    record_object_store_name: String,

    #[envconfig(from = "MINIO_URL")]
    pub minio_url: String,

    #[envconfig(from = "MINIO_REGION", default = "")]
    pub minio_region: String,

    #[envconfig(from = "MINIO_ACCESS_KEY")]
    pub minio_access_key: Option<String>,

    #[envconfig(from = "MINIO_SECRET_KEY")]
    pub minio_secret_key: Option<String>,

    #[envconfig(from = "MINIO_RECORD_BUCKET_NAME", default = "kurec-record")]
    pub minio_record_bucket_name: String,

    #[envconfig(from = "MEILISEARCH_URL")]
    pub meilisearch_url: String,

    #[envconfig(from = "MEILISEARCH_API_KEY")]
    pub meilisearch_api_key: Option<String>,
}

impl KurecConfig {
    pub fn get_config() -> Result<Self, envconfig::Error> {
        KurecConfig::init_from_env()
    }

    pub fn get_epg_bucket_name(&self) -> String {
        format!("{}-{}", self.prefix, self.epg_kv_bucket_name)
    }

    pub fn get_record_bucket_name(&self) -> String {
        format!("{}-{}", self.prefix, self.record_kv_bucket_name)
    }

    pub fn get_epg_stream_name(&self) -> String {
        format!("{}-{}", self.prefix, self.epg_stream_name)
    }

    pub fn get_record_stream_name(&self) -> String {
        format!("{}-{}", self.prefix, self.record_stream_name)
    }

    pub fn get_record_object_store_name(&self) -> String {
        format!("{}-{}", self.prefix, self.record_object_store_name)
    }
}

#[derive(Clone, Debug, Envconfig)]
pub struct NatsRecordKvBucketConfig {}

#[derive(Clone, Debug, Envconfig)]
pub struct NatsStreamConfig {
    pub name: String,
}

#[derive(Clone, Debug, Envconfig)]
pub struct MinioConfig {
    pub region: String,
    pub endpoint_url: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Clone, Debug, Envconfig)]
pub struct MeilisearchConfig {
    pub url: String,
}

#[derive(Clone)]
pub struct MyDuration(Duration);
impl std::ops::Deref for MyDuration {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MyDuration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Debug for MyDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for MyDuration {
    type Err = duration_string::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let d = DurationString::try_from(String::from(s))?.into();
        Ok(MyDuration(d))
    }
}

impl From<MyDuration> for Duration {
    fn from(val: MyDuration) -> Self {
        val.0
    }
}
