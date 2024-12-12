# WebRecordingSchedule

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**failed_reason** | Option<[**models::RecordingFailedReason**](RecordingFailedReason.md)> | Reason of the recording failure.  This property exists only when the recording failed. | [optional]
**options** | [**models::RecordingOptions**](RecordingOptions.md) | Recording options. | 
**program** | [**models::MirakurunProgram**](MirakurunProgram.md) | Metadata of the target TV program. | 
**state** | [**models::RecordingScheduleState**](RecordingScheduleState.md) | The current state of the recording schedule. | 
**tags** | **Vec<String>** | A list of tags. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


