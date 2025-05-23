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
pub struct MirakurunProgramAudio {
    #[serde(rename = "componentType")]
    pub component_type: i32,
    #[serde(rename = "isMain")]
    pub is_main: bool,
    #[serde(rename = "langs")]
    pub langs: Vec<String>,
    #[serde(rename = "samplingRate")]
    pub sampling_rate: i32,
}

impl MirakurunProgramAudio {
    pub fn new(
        component_type: i32,
        is_main: bool,
        langs: Vec<String>,
        sampling_rate: i32,
    ) -> MirakurunProgramAudio {
        MirakurunProgramAudio {
            component_type,
            is_main,
            langs,
            sampling_rate,
        }
    }
}
