use kurec_interface::KurecConfig;
use mirakc_client::models::{self, MirakurunService};

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
