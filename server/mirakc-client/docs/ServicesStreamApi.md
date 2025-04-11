# \ServicesStreamApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**check_service_stream**](ServicesStreamApi.md#check_service_stream) | **HEAD** /services/{id}/stream | 
[**get_service_stream**](ServicesStreamApi.md#get_service_stream) | **GET** /services/{id}/stream | Gets a media stream of a service.



## check_service_stream

> check_service_stream(id, x_mirakurun_priority, decode, pre_filters, post_filters)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | Mirakurun service ID | [required] |
**x_mirakurun_priority** | Option<**i32**> | Priority of the tuner user |  |
**decode** | Option<**bool**> | `0` or `false` disables decoding.  The stream will be decoded by default if a decoder is specified in the `config.yml`. |  |
**pre_filters** | Option<[**Vec<String>**](String.md)> | A list of pre-filters to use. |  |
**post_filters** | Option<[**Vec<String>**](String.md)> | A list of post-filters to use. |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_service_stream

> get_service_stream(id, x_mirakurun_priority, decode, pre_filters, post_filters)
Gets a media stream of a service.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | Mirakurun service ID | [required] |
**x_mirakurun_priority** | Option<**i32**> | Priority of the tuner user |  |
**decode** | Option<**bool**> | `0` or `false` disables decoding.  The stream will be decoded by default if a decoder is specified in the `config.yml`. |  |
**pre_filters** | Option<[**Vec<String>**](String.md)> | A list of pre-filters to use. |  |
**post_filters** | Option<[**Vec<String>**](String.md)> | A list of post-filters to use. |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

