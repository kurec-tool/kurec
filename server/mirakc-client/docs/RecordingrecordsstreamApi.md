# \RecordingrecordsstreamApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**check_record_stream**](RecordingrecordsstreamApi.md#check_record_stream) | **HEAD** /recording/records/{id}/stream | 
[**get_record_stream**](RecordingrecordsstreamApi.md#get_record_stream) | **GET** /recording/records/{id}/stream | Gets a media stream of the content of a record.



## check_record_stream

> check_record_stream(id, post_filters)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Record ID | [required] |
**post_filters** | Option<[**Vec<String>**](String.md)> | post-filters |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_record_stream

> get_record_stream(id, post_filters)
Gets a media stream of the content of a record.

It's possible to get a media stream of the record even while it's recording.  In this case, data will be sent when data is appended to the content file event if the stream reaches EOF at that point.  The streaming will stop within 2 seconds after the stream reaches the *true* EOF.  A request for a record without content file always returns status code 204.  A range request with filters always causes an error response with status code 400.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Record ID | [required] |
**post_filters** | Option<[**Vec<String>**](String.md)> | post-filters |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

