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

/// RecordingStatus : A recording status.
/// A recording status.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecordingStatus {
    String(String),
    RecordingStatusOneOf(Box<models::RecordingStatusOneOf>),
}

impl Default for RecordingStatus {
    fn default() -> Self {
        Self::String(Default::default())
    }
}
