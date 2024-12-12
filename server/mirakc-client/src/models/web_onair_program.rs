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

/// WebOnairProgram : Metadata of TV program that is now on-air in a service.  The metadata is collected from EIT[p/f] sections, not from EIT[schedule] sections.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct WebOnairProgram {
    /// A TV program that is now on-air.  `null` when no TV program is broadcasted.
    #[serde(rename = "current", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub current: Option<Option<Box<models::MirakurunProgram>>>,
    /// A TV program that will start next.  `null` when there is no next TV program.
    #[serde(rename = "next", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub next: Option<Option<Box<models::MirakurunProgram>>>,
    /// Mirakurun service ID.
    #[serde(rename = "serviceId")]
    pub service_id: i64,
}

impl WebOnairProgram {
    /// Metadata of TV program that is now on-air in a service.  The metadata is collected from EIT[p/f] sections, not from EIT[schedule] sections.
    pub fn new(service_id: i64) -> WebOnairProgram {
        WebOnairProgram {
            current: None,
            next: None,
            service_id,
        }
    }
}
