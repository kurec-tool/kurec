use kurec_interface::MirakurunService;
use kurec_interface::{KurecConfig, MirakurunService};
use tokio;

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
        service_id: u64,
    ) -> Result<MirakurunService, anyhow::Error> {
        let service_url = format!("{}/api/services/{}", tuner_url, service_id);
        let resp = reqwest::Client::new()
            .get(&service_url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        let service = resp.json::<MirakurunService>().await?;
        Ok(service)
    }

    pub async fn get_programs(&self, tuner_url: &str, service_id: u64) {
        let programs_url = format!("{}/api/services/{}/programs", tuner_url, service_id);
        todo!()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_service_success() {
        let mut server = mockito::Server::new_async().await;

        let _m = server
            .mock("GET", "/api/services/3333311111")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "id": 3333311111,
                "serviceId": 11111,
                "networkId": 33333,
                "type": 1,
                "logoId": 11,
                "remoteControlKeyId": 1,
                "name": "テストサービス局",
                "channel": {
                    "type": "GR",
                    "channel": "T11"
                },
                "hasLogoData": false
            }"#,
            )
            .create();

        let config = KurecConfig::default();
        let adapter = MirakcAdapter::new(config);
        let result = adapter.get_service(&server.url(), 1234).await;

        assert!(result.is_ok());
        let service = result.unwrap();
        assert_eq!(service.id, 1234);
        assert_eq!(service.name, "Test Service");
    }

    #[tokio::test]
    async fn test_get_service_not_found() {
        let _m = mock("GET", "/api/services/1234").with_status(404).create();

        let config = KurecConfig::default();
        let adapter = MirakcAdapter::new(config);
        let result = adapter.get_service(&mockito::server_url(), 1234).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_service_invalid_response() {
        let _m = mock("GET", "/api/services/1234")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"invalid": "response"}"#)
            .create();

        let config = KurecConfig::default();
        let adapter = MirakcAdapter::new(config);
        let result = adapter.get_service(&mockito::server_url(), 1234).await;

        assert!(result.is_err());
    }
}
