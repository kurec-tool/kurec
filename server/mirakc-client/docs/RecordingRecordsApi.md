# \RecordingRecordsApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_record**](RecordingRecordsApi.md#get_record) | **GET** /recording/records/{id} | Gets metadata of a record.
[**get_records**](RecordingRecordsApi.md#get_records) | **GET** /recording/records | Lists records.
[**remove_record**](RecordingRecordsApi.md#remove_record) | **DELETE** /recording/records/{id} | Removes a record.



## get_record

> models::WebRecord get_record(id)
Gets metadata of a record.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Record ID | [required] |

### Return type

[**models::WebRecord**](WebRecord.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_records

> Vec<models::WebRecord> get_records()
Lists records.

The following kind of records are also listed:  * Records currently recording * Records failed recording but have recorded data * Records that have no content files (maybe, those were removed outside the system)  

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::WebRecord>**](WebRecord.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## remove_record

> remove_record(id, purge)
Removes a record.

The record cannot be removed while it's recording.  Firstly stop the recording, then remove.  The record can be removed even while streaming its content.  In this case, the streaming will stop once the buffered data has been sent.  The content file of the record is removed together with the record if the `purge` query parameter is specified.  The log file is also removed if it exists.  A `recording.record-removed` event will be sent if the record is removed successfully.  A `recording.content-removed` event will be sent if the content file of the record is removed successfully.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Record ID | [required] |
**purge** | Option<**bool**> | `1` or `true` will purge the content file.  The content file won't be purged by default. |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

