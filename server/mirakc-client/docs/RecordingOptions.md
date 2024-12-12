# RecordingOptions

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**content_path** | Option<**String**> | The path of the content file relative to `config.recording.basedir`.  The path must be a valid Unicode string.  ### If `config.recording.records-dir` is NOT specified  This is a required option.  A response with the status code 401 will be replied if this option is not specified.  ### If `config.recording.records-dir` is specified  An auto-generated filename will be used for the content file if this option is not specified. | [optional]
**post_filters** | Option<**Vec<String>**> | A list of post-filters to use. | [optional]
**pre_filters** | Option<**Vec<String>**> | A list of pre-filters to use. | [optional]
**priority** | Option<**i32**> | A priority of tuner usage. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


