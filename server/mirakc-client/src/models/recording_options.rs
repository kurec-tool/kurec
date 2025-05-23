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

/// RecordingOptions : Recording options.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordingOptions {
    /// The path of the content file relative to `config.recording.basedir`.  The path must be a valid Unicode string.  ### If `config.recording.records-dir` is NOT specified  This is a required option.  A response with the status code 401 will be replied if this option is not specified.  ### If `config.recording.records-dir` is specified  An auto-generated filename will be used for the content file if this option is not specified.
    #[serde(
        rename = "contentPath",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub content_path: Option<Option<String>>,
    /// Log filter of the recording schedule.  If this option is not specified, the value of `config.recording.log-filter` is used when the recording starts.  The log filter will be set to the environment variable `MIRAKC_ARIB_LOG` passed to the recording command pipeline.  If the log filter is neither empty nor `off`, logs coming from the recording command pipeline will be stored into a log file.  The log file will be placed in the same folder as the content file and its name is \"<content-file>.log\".  The environment variable `MIRAKC_ARIB_LOG_NO_TIMESTAMP` will be disabled and each log will have a timestamp.  If the log filter is empty or `off`, no log will come from the recording command pipeline. And the log file won't be created.  For backward compatibility with 3.x and older versions, the logs will be output to STDOUT if neither this option nor `recording.log-filter` is specified.
    #[serde(
        rename = "logFilter",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub log_filter: Option<Option<String>>,
    /// A list of post-filters to use.
    #[serde(rename = "postFilters", skip_serializing_if = "Option::is_none")]
    pub post_filters: Option<Vec<String>>,
    /// A list of pre-filters to use.
    #[serde(rename = "preFilters", skip_serializing_if = "Option::is_none")]
    pub pre_filters: Option<Vec<String>>,
    /// A priority of tuner usage.
    #[serde(rename = "priority", skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
}

impl RecordingOptions {
    /// Recording options.
    pub fn new() -> RecordingOptions {
        RecordingOptions {
            content_path: None,
            log_filter: None,
            post_filters: None,
            pre_filters: None,
            priority: None,
        }
    }
}
