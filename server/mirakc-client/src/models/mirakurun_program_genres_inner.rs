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
pub struct MirakurunProgramGenresInner {
    #[serde(rename = "lv1")]
    pub lv1: i32,
    #[serde(rename = "lv2")]
    pub lv2: i32,
    #[serde(rename = "un1")]
    pub un1: i32,
    #[serde(rename = "un2")]
    pub un2: i32,
}

impl MirakurunProgramGenresInner {
    pub fn new(lv1: i32, lv2: i32, un1: i32, un2: i32) -> MirakurunProgramGenresInner {
        MirakurunProgramGenresInner { lv1, lv2, un1, un2 }
    }
}
