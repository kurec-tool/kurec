# WebTimeshiftRecorder

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**current_record_id** | Option<**i32**> | An ID of the record currently being recorded. | [optional]
**duration** | **i64** | The duration of the timeshift timeline.  `0` when there is no record. | 
**end_time** | Option<**i64**> | The end time of the timeshift timeline.  `null` when there is no record. | [optional]
**name** | **String** | The timeshift recorder name defined in `config.yml`. | 
**num_records** | **i32** | The number of records available for playback.  The number will change over the recording.  For example, [/timeshift/{recorder}/records](#/timeshift::records/getTimeshiftRecords) may return different number of records from this value. | 
**pipeline** | [**Vec<models::WebProcessModel>**](WebProcessModel.md) | A list of process models constituting the timeshift pipeline currently running. | 
**recording** | **bool** | `true` while recording, `false` otherwise.  Users can still access the records even if this property returns `false`. | 
**service** | [**models::MirakurunService**](MirakurunService.md) | Metadata of the service to be recorded. | 
**start_time** | Option<**i64**> | The start time of the timeshift timeline.  `null` when there is no record. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


