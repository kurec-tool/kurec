# RecordingOptions

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**content_path** | Option<**String**> | The path of the content file relative to `config.recording.basedir`.  The path must be a valid Unicode string.  ### If `config.recording.records-dir` is NOT specified  This is a required option.  A response with the status code 401 will be replied if this option is not specified.  ### If `config.recording.records-dir` is specified  An auto-generated filename will be used for the content file if this option is not specified. | [optional]
**log_filter** | Option<**String**> | Log filter of the recording schedule.  If this option is not specified, the value of `config.recording.log-filter` is used when the recording starts.  The log filter will be set to the environment variable `MIRAKC_ARIB_LOG` passed to the recording command pipeline.  If the log filter is neither empty nor `off`, logs coming from the recording command pipeline will be stored into a log file.  The log file will be placed in the same folder as the content file and its name is \"<content-file>.log\".  The environment variable `MIRAKC_ARIB_LOG_NO_TIMESTAMP` will be disabled and each log will have a timestamp.  If the log filter is empty or `off`, no log will come from the recording command pipeline. And the log file won't be created.  For backward compatibility with 3.x and older versions, the logs will be output to STDOUT if neither this option nor `recording.log-filter` is specified. | [optional]
**post_filters** | Option<**Vec<String>**> | A list of post-filters to use. | [optional]
**pre_filters** | Option<**Vec<String>**> | A list of pre-filters to use. | [optional]
**priority** | Option<**i32**> | A priority of tuner usage. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


