/*
 * mirakc Web API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 3.3.0-dev.0
 *
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct MirakurunService {
    #[serde(rename = "channel")]
    pub channel: Box<models::MirakurunServiceChannel>,
    #[serde(rename = "hasLogoData")]
    pub has_logo_data: bool,
    #[serde(rename = "id")]
    pub id: i64,
    #[serde(rename = "logoId", skip_serializing_if = "Option::is_none")]
    pub logo_id: Option<i32>,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "networkId")]
    pub network_id: i32,
    #[serde(rename = "remoteControlKeyId", skip_serializing_if = "Option::is_none")]
    pub remote_control_key_id: Option<i32>,
    #[serde(rename = "serviceId")]
    pub service_id: i32,
    #[serde(rename = "type")]
    pub r#type: i32,
}

impl MirakurunService {
    pub fn new(
        channel: models::MirakurunServiceChannel,
        has_logo_data: bool,
        id: i64,
        name: String,
        network_id: i32,
        service_id: i32,
        r#type: i32,
    ) -> MirakurunService {
        MirakurunService {
            channel: Box::new(channel),
            has_logo_data,
            id,
            logo_id: None,
            name,
            network_id,
            remote_control_key_id: None,
            service_id,
            r#type,
        }
    }
}
