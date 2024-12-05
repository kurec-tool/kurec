use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::model::mirakurun::{
    program::{Program, Programs},
    service::Service,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonProgram {
    pub id: u64,
    pub json: String,
}

pub async fn get_json_programs(
    tuner_url: &str,
    service_id: u64,
) -> Result<Vec<JsonProgram>, anyhow::Error> {
    let programs_url = format!("{}/api/services/{}/programs", tuner_url, service_id);
    let resp = reqwest::get(&programs_url).await?;
    if resp.status() != 200 {
        return Err(anyhow::anyhow!(
            "url: {} status: {}",
            programs_url,
            resp.status()
        ));
    }
    let bytes = resp.bytes().await.map_err(anyhow::Error::new);
    let json_array: Vec<serde_json::Value> = serde_json::from_slice(&bytes?)?;
    let json_programs: Vec<JsonProgram> = json_array
        .into_iter()
        .map(|json| JsonProgram {
            id: json["id"].as_u64().unwrap_or_default(),
            json: json.to_string(),
        })
        .filter(|json_program| json_program.id != 0)
        .collect();
    Ok(json_programs)
}

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

pub async fn get_program(tuner_url: &str, program_id: u64) -> Result<Program, anyhow::Error> {
    let programs_url = format!("{}/api/programs/{}", tuner_url, program_id);
    let resp = reqwest::get(&programs_url).await?;
    if resp.status() != 200 {
        return Err(anyhow::anyhow!(
            "url: {} status: {}",
            programs_url,
            resp.status()
        ));
    }
    resp.json::<Program>().await.map_err(anyhow::Error::new)
}

pub async fn list_programs_by_service_id(
    tuner_url: &str,
    service_id: u64,
) -> Result<Vec<Program>, anyhow::Error> {
    let programs_url = format!("{}/api/services/{}/programs", tuner_url, service_id);
    let resp = reqwest::get(&programs_url).await?;
    if resp.status() != 200 {
        return Err(anyhow::anyhow!(
            "url: {} status: {}",
            programs_url,
            resp.status()
        ));
    }
    resp.json::<Vec<Program>>()
        .await
        .map_err(anyhow::Error::new)
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

pub async fn get_service_by_network_id_and_service_id(
    tuner_url: &str,
    network_id: u64,
    service_id: u64,
) -> Result<Service, anyhow::Error> {
    get_service(tuner_url, network_id * 100000 + service_id).await
}

pub async fn list_scheduled_program_ids(tuner_url: &str) -> Result<Vec<u64>, anyhow::Error> {
    let schedules_url = format!("{}/api/recording/schedules", tuner_url);
    let resp = reqwest::get(&schedules_url).await?;
    if resp.status() != 200 {
        return Err(anyhow::anyhow!(
            "url: {} status: {}",
            schedules_url,
            resp.status()
        ));
    }
    let json_array: Vec<serde_json::Value> = resp.json().await?;
    let program_ids: Vec<u64> = json_array
        .into_iter()
        .map(|json| json["program"]["id"].as_u64().unwrap_or_default())
        .filter(|program_id| *program_id != 0)
        .collect();
    Ok(program_ids)
}

pub async fn schedule_program(tuner_url: &str, program_id: u64) -> Result<(), anyhow::Error> {
    let schedule_url = format!("{}/api/recording/schedules/{}", tuner_url, program_id);
    let client = reqwest::Client::new();
    let resp = client.get(&schedule_url).send().await?;
    if resp.status() == 200 {
        // 見つかったので予約操作しない
        return Ok(());
    }

    let schedules_url = format!("{}/api/recording/schedules", tuner_url);
    let body =
        json!({"programId": program_id, "options": {"contentPath": format!("{}.ts", program_id)}});
    let resp = client.post(&schedules_url).json(&body).send().await?;
    if resp.status() != 201 {
        return Err(anyhow::anyhow!(
            "url: {} status: {}",
            schedules_url,
            resp.status()
        ));
    }
    Ok(())
}

pub async fn unschedule_program(tuner_url: &str, program_id: u64) -> Result<(), anyhow::Error> {
    let schedule_url = format!("{}/api/recording/schedules/{}", tuner_url, program_id);
    let client = reqwest::Client::new();
    let resp = client.get(&schedule_url).send().await?;
    if resp.status() != 200 {
        // 見つからなかったので予約操作しない
        return Ok(());
    }

    let resp = client.delete(&schedule_url).send().await?;
    if resp.status() != 200 {
        return Err(anyhow::anyhow!(
            "url: {} status: {}",
            schedule_url,
            resp.status()
        ));
    }
    Ok(())
}
