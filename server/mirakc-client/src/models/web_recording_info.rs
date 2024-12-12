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

/// WebRecordingInfo : A recording information model.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct WebRecordingInfo {
    /// The duration of the **actual** recording in milliseconds.  The value may not equal to the duration of the TV program.  Undefined during recording.
    #[serde(rename = "duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
    /// The end time of the **actual** recording in UNIX time (milliseconds).  The value may not equal to the end time of the TV program.  Undefined during recording.
    #[serde(rename = "endTime", skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    /// Recording options.
    #[serde(rename = "options")]
    pub options: Box<models::RecordingOptions>,
    /// The start time of the **actual** recording in UNIX time (milliseconds).  The value may not equal to the start time of the TV program.
    #[serde(rename = "startTime")]
    pub start_time: i64,
    /// The current status of the record.
    #[serde(rename = "status")]
    pub status: Box<models::RecordingStatus>,
}

impl WebRecordingInfo {
    /// A recording information model.
    pub fn new(options: models::RecordingOptions, start_time: i64, status: models::RecordingStatus) -> WebRecordingInfo {
        WebRecordingInfo {
            duration: None,
            end_time: None,
            options: Box::new(options),
            start_time,
            status: Box::new(status),
        }
    }
}
