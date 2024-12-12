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
pub struct RecordingFailedReasonOneOf4 {
    #[serde(rename = "type")]
    pub r#type: Type,
}

impl RecordingFailedReasonOneOf4 {
    pub fn new(r#type: Type) -> RecordingFailedReasonOneOf4 {
        RecordingFailedReasonOneOf4 {
            r#type,
        }
    }
}
/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "schedule-expired")]
    ScheduleExpired,
}

impl Default for Type {
    fn default() -> Type {
        Self::ScheduleExpired
    }
}
