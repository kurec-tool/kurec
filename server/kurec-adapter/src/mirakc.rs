use std::io::Write;

use futures::StreamExt;
use kurec_interface::KurecConfig;
use mirakc_client::models::{self, MirakurunService};
use tracing::debug;

#[derive(Clone, Debug)]
pub struct MirakcAdapter {
    pub config: KurecConfig,
}

impl MirakcAdapter {
    pub fn new(config: KurecConfig) -> Self {
        Self { config }
    }

    pub async fn get_service(
        &self,
        tuner_url: &str,
        service_id: i64,
    ) -> Result<MirakurunService, anyhow::Error> {
        let config = mirakc_client::apis::configuration::Configuration {
            base_path: format!("{}/api", tuner_url),
            ..Default::default()
        };
        let ret = mirakc_client::apis::services_api::get_service(&config, service_id).await?;
        Ok(ret)
    }

    pub async fn get_programs(
        &self,
        tuner_url: &str,
        service_id: i64,
    ) -> Result<Vec<models::MirakurunProgram>, anyhow::Error> {
        let config = mirakc_client::apis::configuration::Configuration {
            base_path: format!("{}/api", tuner_url),
            ..Default::default()
        };
        let ret =
            mirakc_client::apis::services_api::get_programs_of_service(&config, service_id).await?;
        Ok(ret)
    }

    pub async fn get_record(
        &self,
        tuner_url: &str,
        record_id: &str,
    ) -> Result<models::WebRecord, anyhow::Error> {
        debug!("get_record: {}", record_id);
        let config = mirakc_client::apis::configuration::Configuration {
            base_path: format!("{}/api", tuner_url),
            ..Default::default()
        };
        let ret = mirakc_client::apis::recording_records_api::get_record(&config, record_id)
            .await
            .unwrap();
        Ok(ret)
    }

    pub async fn save_record_stream<T: Write>(
        &self,
        tuner_url: &str,
        record_id: &str,
        write_to: &mut T,
    ) -> Result<(), anyhow::Error> {
        debug!(
            "save_record_stream: tuner: {}, id: {}",
            tuner_url, record_id
        );
        let url = format!("{}/api/recording/records/{}/stream", tuner_url, record_id);
        let resp = reqwest::Client::new().get(url).send().await.unwrap();
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to save record stream: {}",
                resp.status()
            ));
        }
        debug!("save_record_stream: status: {:?}", resp);
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let data = chunk.unwrap();
            write_to.write_all(data.as_ref()).unwrap();
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_service() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/api/services/1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "id": 3333311111,
                    "serviceId": 11111,
                    "networkId": 33333,
                    "type": 1,
                    "logoId": 13,
                    "remoteControlKeyId": 13,
                    "name": "テストサービス局",
                    "channel": {
                        "type": "GR",
                        "channel": "T13"
                    },
                    "hasLogoData": false
                }"#,
            )
            .create();

        let config = KurecConfig::default();
        let adapter = MirakcAdapter::new(config);
        dbg!(&server.url());
        let result = adapter.get_service(&server.url(), 1).await;

        dbg!(&result);

        assert!(result.is_ok());
        let service = result.unwrap();
        assert_eq!(service.id, 3333311111);
        assert_eq!(service.name, "テストサービス局");
    }

    #[tokio::test]
    async fn test_get_service_not_found() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/api/services/999")
            .with_status(404)
            .create();

        let config = KurecConfig::default();
        let adapter = MirakcAdapter::new(config);
        let result = adapter.get_service(&server.url(), 999).await;

        assert!(result.is_err());
    }
}
