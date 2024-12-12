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
pub struct RecordingStatusOneOfFailed {
    #[serde(rename = "reason")]
    pub reason: Box<models::RecordingFailedReason>,
}

impl RecordingStatusOneOfFailed {
    pub fn new(reason: models::RecordingFailedReason) -> RecordingStatusOneOfFailed {
        RecordingStatusOneOfFailed {
            reason: Box::new(reason),
        }
    }
}

