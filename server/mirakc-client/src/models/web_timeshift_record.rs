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

/// WebTimeshiftRecord : Metadata of a timeshift record.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct WebTimeshiftRecord {
    /// The duration of the timeshift record in milliseconds.
    #[serde(rename = "duration")]
    pub duration: i64,
    /// A timeshift record ID.
    #[serde(rename = "id")]
    pub id: i32,
    /// Metadata of the TV program.
    #[serde(rename = "program")]
    pub program: Box<models::MirakurunProgram>,
    /// `true` while recording, `false` otherwise.
    #[serde(rename = "recording")]
    pub recording: bool,
    /// The size of the timeshift record in bytes.
    #[serde(rename = "size")]
    pub size: i64,
    /// The start time of the timeshift record in UNIX time (milliseconds).
    #[serde(rename = "startTime")]
    pub start_time: i64,
}

impl WebTimeshiftRecord {
    /// Metadata of a timeshift record.
    pub fn new(duration: i64, id: i32, program: models::MirakurunProgram, recording: bool, size: i64, start_time: i64) -> WebTimeshiftRecord {
        WebTimeshiftRecord {
            duration,
            id,
            program: Box::new(program),
            recording,
            size,
            start_time,
        }
    }
}

