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
pub struct MirakurunProgramVideo {
    #[serde(rename = "componentType")]
    pub component_type: i32,
    #[serde(
        rename = "resolution",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub resolution: Option<Option<String>>,
    #[serde(rename = "streamContent")]
    pub stream_content: i32,
    #[serde(
        rename = "type",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub r#type: Option<Option<String>>,
}

impl MirakurunProgramVideo {
    pub fn new(component_type: i32, stream_content: i32) -> MirakurunProgramVideo {
        MirakurunProgramVideo {
            component_type,
            resolution: None,
            stream_content,
            r#type: None,
        }
    }
}
