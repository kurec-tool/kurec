# \RecordingschedulesApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_recording_schedule**](RecordingschedulesApi.md#create_recording_schedule) | **POST** /recording/schedules | Books a recording schedule.
[**delete_recording_schedule**](RecordingschedulesApi.md#delete_recording_schedule) | **DELETE** /recording/schedules/{program_id} | Deletes a recording schedule.
[**delete_recording_schedules**](RecordingschedulesApi.md#delete_recording_schedules) | **DELETE** /recording/schedules | Clears recording schedules.
[**get_recording_schedule**](RecordingschedulesApi.md#get_recording_schedule) | **GET** /recording/schedules/{program_id} | Gets a recording schedule.
[**get_recording_schedules**](RecordingschedulesApi.md#get_recording_schedules) | **GET** /recording/schedules | Lists recording schedules.



## create_recording_schedule

> models::WebRecordingSchedule create_recording_schedule(web_recording_schedule_input)
Books a recording schedule.

### If `config.recording.records-dir` is specified  When the recording starts, a record will be created in the specified folder and a `recording.record-saved` event will be sent if the record is created successfully.  Otherwise, a `recording.record-broken` event will be sent instead.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**web_recording_schedule_input** | [**WebRecordingScheduleInput**](WebRecordingScheduleInput.md) |  | [required] |

### Return type

[**models::WebRecordingSchedule**](WebRecordingSchedule.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_recording_schedule

> delete_recording_schedule(program_id)
Deletes a recording schedule.

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


## delete_recording_schedules

> delete_recording_schedules(tag)
Clears recording schedules.

If a tag name is specified in the `tag` query parameter, recording schedules tagged with the specified name will be deleted.  Otherwise, all recording schedules will be deleted.  When deleting recording schedules by a tag, recording schedules that meet any of the following conditions won't be deleted:    * Recording schedules without the specified tag   * Recording schedules in the `tracking` or `recording` state   * Recording schedules in the `scheduled` state and will start recording     soon

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tag** | Option<**String**> | Tag name |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_recording_schedule

> models::WebRecordingSchedule get_recording_schedule(program_id)
Gets a recording schedule.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**program_id** | **i64** | Mirakurun program ID | [required] |

### Return type

[**models::WebRecordingSchedule**](WebRecordingSchedule.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_recording_schedules

> Vec<models::WebRecordingSchedule> get_recording_schedules()
Lists recording schedules.

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::WebRecordingSchedule>**](WebRecordingSchedule.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

