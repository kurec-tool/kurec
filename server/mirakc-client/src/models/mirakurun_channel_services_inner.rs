/*
 * mirakc Web API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 4.0.0-dev.0
 *
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct MirakurunChannelServicesInner {
    #[serde(rename = "id")]
    pub id: i64,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "networkId")]
    pub network_id: i32,
    #[serde(rename = "serviceId")]
    pub service_id: i32,
}

impl MirakurunChannelServicesInner {
    pub fn new(
        id: i64,
        name: String,
        network_id: i32,
        service_id: i32,
    ) -> MirakurunChannelServicesInner {
        MirakurunChannelServicesInner {
            id,
            name,
            network_id,
            service_id,
        }
    }
}
