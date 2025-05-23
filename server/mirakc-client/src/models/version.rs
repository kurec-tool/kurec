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

/// Version : Version information of mirakc currently running.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Version {
    /// Current version.
    #[serde(rename = "current")]
    pub current: String,
    /// Same as `current`.
    #[serde(rename = "latest")]
    pub latest: String,
}

impl Version {
    /// Version information of mirakc currently running.
    pub fn new(current: String, latest: String) -> Version {
        Version { current, latest }
    }
}
