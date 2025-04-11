# WebRecordingInfo

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**duration** | Option<**i64**> | The duration of the **actual** recording in milliseconds.  The value may not equal to the duration of the TV program.  Undefined during recording. | [optional]
**end_time** | Option<**i64**> | The end time of the **actual** recording in UNIX time (milliseconds).  The value may not equal to the end time of the TV program.  Undefined during recording. | [optional]
**failed_reason** | Option<[**models::RecordingFailedReason**](RecordingFailedReason.md)> | The reason for the recording failure.  This property is available only when the `status` is `failed`. | [optional]
**options** | [**models::RecordingOptions**](RecordingOptions.md) | Recording options. | 
**start_time** | **i64** | The start time of the **actual** recording in UNIX time (milliseconds).  The value may not equal to the start time of the TV program. | 
**status** | [**models::WebRecordingStatus**](WebRecordingStatus.md) | The current status of the record. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


