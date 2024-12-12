# WebContentInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**length** | Option<**i64**> | The size of the content.  `null` if there is no content file at the location specified by `content_path` of the recording schedule.  `0` will be set if failed getting the size of the content file even though the file exists. | [optional]
**path** | **String** | The path of the content file relative to `config.recording.basedir`. | 
**r#type** | **String** | The MIME type of the content. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


