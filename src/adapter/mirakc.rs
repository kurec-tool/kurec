use serde_json::json;

use crate::model::mirakurun::{program::Programs, service::Service};

pub async fn get_programs(tuner_url: &str, service_id: u64) -> Result<Programs, anyhow::Error> {
    let programs_url = format!("{}/api/services/{}/programs", tuner_url, service_id);
    let resp = reqwest::get(&programs_url).await?;
    if resp.status() != 200 {
        return Err(anyhow::anyhow!(
            "url: {} status: {}",
            programs_url,
            resp.status()
        ));
    }
    resp.json::<Programs>().await.map_err(anyhow::Error::new)
}

pub async fn get_service(tuner_url: &str, service_id: u64) -> Result<Service, anyhow::Error> {
    let service_url = format!("{}/api/services/{}", tuner_url, service_id);
    let resp = reqwest::get(&service_url).await?;
    if resp.status() != 200 {
        return Err(anyhow::anyhow!(
            "url: {} status: {}",
            service_url,
            resp.status()
        ));
    }
    resp.json::<Service>().await.map_err(anyhow::Error::new)
}

pub async fn schedule_program(tuner_url: &str, program_id: &u64) -> Result<(), anyhow::Error> {
    let schedule_url = format!("{}/api/recording/schedules", tuner_url);
    let body =
        json!({"programId": program_id, "options": {"contentPath": format!("{}.ts", program_id)}});
    let client = reqwest::Client::new();
    let resp = client.post(&schedule_url).json(&body).send().await?;
    if resp.status() != 201 {
        return Err(anyhow::anyhow!(
            "url: {} status: {}",
            schedule_url,
            resp.status()
        ));
    }
    Ok(())
}
