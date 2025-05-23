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
pub struct MirakurunProgram {
    #[serde(
        rename = "audio",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub audio: Option<Option<Box<models::MirakurunProgramAudio>>>,
    #[serde(rename = "audios", skip_serializing_if = "Option::is_none")]
    pub audios: Option<Vec<models::MirakurunProgramAudiosInner>>,
    #[serde(
        rename = "description",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<Option<String>>,
    #[serde(rename = "duration")]
    pub duration: i64,
    #[serde(rename = "eventId")]
    pub event_id: i32,
    #[serde(rename = "extended", skip_serializing_if = "Option::is_none")]
    pub extended: Option<serde_json::Value>,
    #[serde(
        rename = "genres",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub genres: Option<Option<Vec<models::MirakurunProgramGenresInner>>>,
    #[serde(rename = "id")]
    pub id: i64,
    #[serde(rename = "isFree")]
    pub is_free: bool,
    #[serde(
        rename = "name",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub name: Option<Option<String>>,
    #[serde(rename = "networkId")]
    pub network_id: i32,
    #[serde(rename = "relatedItems", skip_serializing_if = "Option::is_none")]
    pub related_items: Option<Vec<models::MirakurunProgramRelatedItemsInner>>,
    #[serde(
        rename = "series",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub series: Option<Option<Box<models::MirakurunProgramSeries>>>,
    #[serde(rename = "serviceId")]
    pub service_id: i32,
    #[serde(rename = "startAt")]
    pub start_at: i64,
    #[serde(
        rename = "video",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub video: Option<Option<Box<models::MirakurunProgramVideo>>>,
}

impl MirakurunProgram {
    pub fn new(
        duration: i64,
        event_id: i32,
        id: i64,
        is_free: bool,
        network_id: i32,
        service_id: i32,
        start_at: i64,
    ) -> MirakurunProgram {
        MirakurunProgram {
            audio: None,
            audios: None,
            description: None,
            duration,
            event_id,
            extended: None,
            genres: None,
            id,
            is_free,
            name: None,
            network_id,
            related_items: None,
            series: None,
            service_id,
            start_at,
            video: None,
        }
    }
}
