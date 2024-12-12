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
pub struct MirakurunServiceChannel {
    #[serde(rename = "channel")]
    pub channel: String,
    #[serde(rename = "type")]
    pub r#type: models::ChannelType,
}

impl MirakurunServiceChannel {
    pub fn new(channel: String, r#type: models::ChannelType) -> MirakurunServiceChannel {
        MirakurunServiceChannel {
            channel,
            r#type,
        }
    }
}
