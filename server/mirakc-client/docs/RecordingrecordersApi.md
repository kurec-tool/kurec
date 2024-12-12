# \RecordingrecordersApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_recorder**](RecordingrecordersApi.md#get_recorder) | **GET** /recording/recorders/{program_id} | Gets a recorder.
[**get_recorders**](RecordingrecordersApi.md#get_recorders) | **GET** /recording/recorders | Lists recorders.
[**start_recording**](RecordingrecordersApi.md#start_recording) | **POST** /recording/recorders | Starts recording immediately.
[**stop_recording**](RecordingrecordersApi.md#stop_recording) | **DELETE** /recording/recorders/{program_id} | Stops recording.



## get_recorder

> models::WebRecordingRecorder get_recorder(program_id)
Gets a recorder.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**program_id** | **i64** | Mirakurun program ID | [required] |

### Return type

[**models::WebRecordingRecorder**](WebRecordingRecorder.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_recorders

> Vec<models::WebRecordingRecorder> get_recorders()
Lists recorders.

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::WebRecordingRecorder>**](WebRecordingRecorder.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## start_recording

> start_recording(web_recording_schedule_input)
Starts recording immediately.

> [!WARNING] > Use `POST /api/recording/schedules` instead. > The recording starts even if the TV program has not started. > In this case, the recording will always fail.  ### If `config.recording.records-dir` is specified  A record will be created in the specified folder and a `recording.record-saved` event will be sent if the record is created successfully.  Otherwise, a `recording.record-broken` event will be sent instead.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**web_recording_schedule_input** | [**WebRecordingScheduleInput**](WebRecordingScheduleInput.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## stop_recording

> stop_recording(program_id)
Stops recording.

Unlike `DELETE /api/recording/schedules/{program_id}`, this endpoint only stops the recording without removing the corresponding recording schedule.  A `recording.stopped` event will be sent and `GET /api/recording/schedules/{program_id}` will return the schedule information.  ### If `config.recording.records-dir` is specified  A `recording.record-saved` event will be sent if the record is updated successfully. Otherwise, a `recording.record-broken` event will be sent instead.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**program_id** | **i64** | Mirakurun program ID | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

