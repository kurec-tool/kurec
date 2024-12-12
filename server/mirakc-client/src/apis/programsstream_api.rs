/*
 * mirakc Web API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 3.3.0-dev.0
 * 
 * Generated by: https://openapi-generator.tech
 */


use reqwest;
use serde::{Deserialize, Serialize};
use crate::{apis::ResponseContent, models};
use super::{Error, configuration};


/// struct for typed errors of method [`check_program_stream`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CheckProgramStreamError {
    Status404(),
    Status500(),
    Status503(),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_program_stream`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetProgramStreamError {
    Status404(),
    Status500(),
    Status503(),
    UnknownValue(serde_json::Value),
}


pub async fn check_program_stream(configuration: &configuration::Configuration, id: i64, x_mirakurun_priority: Option<i32>, decode: Option<bool>, pre_filters: Option<Vec<String>>, post_filters: Option<Vec<String>>) -> Result<(), Error<CheckProgramStreamError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/programs/{id}/stream", local_var_configuration.base_path, id=id);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::HEAD, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = decode {
        local_var_req_builder = local_var_req_builder.query(&[("decode", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = pre_filters {
        local_var_req_builder = match "multi" {
            "multi" => local_var_req_builder.query(&local_var_str.into_iter().map(|p| ("pre-filters".to_owned(), p.to_string())).collect::<Vec<(std::string::String, std::string::String)>>()),
            _ => local_var_req_builder.query(&[("pre-filters", &local_var_str.into_iter().map(|p| p.to_string()).collect::<Vec<String>>().join(",").to_string())]),
        };
    }
    if let Some(ref local_var_str) = post_filters {
        local_var_req_builder = match "multi" {
            "multi" => local_var_req_builder.query(&local_var_str.into_iter().map(|p| ("post-filters".to_owned(), p.to_string())).collect::<Vec<(std::string::String, std::string::String)>>()),
            _ => local_var_req_builder.query(&[("post-filters", &local_var_str.into_iter().map(|p| p.to_string()).collect::<Vec<String>>().join(",").to_string())]),
        };
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(local_var_param_value) = x_mirakurun_priority {
        local_var_req_builder = local_var_req_builder.header("X-Mirakurun-Priority", local_var_param_value.to_string());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<CheckProgramStreamError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// ### A special hack for EPGStation  If the User-Agent header string starts with \"EPGStation/\", this endpoint creates a temporal on-air program tracker if there is no tracker defined in config.yml, which can be reused for tracking changes of the TV program metadata.  The temporal on-air program tracker will be stopped within 1 minute after the streaming stopped.  The metadata will be returned from [/programs/{id}](#/programs/getProgram).
pub async fn get_program_stream(configuration: &configuration::Configuration, id: i64, x_mirakurun_priority: Option<i32>, decode: Option<bool>, pre_filters: Option<Vec<String>>, post_filters: Option<Vec<String>>) -> Result<(), Error<GetProgramStreamError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/programs/{id}/stream", local_var_configuration.base_path, id=id);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = decode {
        local_var_req_builder = local_var_req_builder.query(&[("decode", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = pre_filters {
        local_var_req_builder = match "multi" {
            "multi" => local_var_req_builder.query(&local_var_str.into_iter().map(|p| ("pre-filters".to_owned(), p.to_string())).collect::<Vec<(std::string::String, std::string::String)>>()),
            _ => local_var_req_builder.query(&[("pre-filters", &local_var_str.into_iter().map(|p| p.to_string()).collect::<Vec<String>>().join(",").to_string())]),
        };
    }
    if let Some(ref local_var_str) = post_filters {
        local_var_req_builder = match "multi" {
            "multi" => local_var_req_builder.query(&local_var_str.into_iter().map(|p| ("post-filters".to_owned(), p.to_string())).collect::<Vec<(std::string::String, std::string::String)>>()),
            _ => local_var_req_builder.query(&[("post-filters", &local_var_str.into_iter().map(|p| p.to_string()).collect::<Vec<String>>().join(",").to_string())]),
        };
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(local_var_param_value) = x_mirakurun_priority {
        local_var_req_builder = local_var_req_builder.header("X-Mirakurun-Priority", local_var_param_value.to_string());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<GetProgramStreamError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

