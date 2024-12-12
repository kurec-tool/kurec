use kurec_interface::KurecConfig;
use kurec_interface::MirakurunService;
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
