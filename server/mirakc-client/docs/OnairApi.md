# \OnairApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_onair_program**](OnairApi.md#get_onair_program) | **GET** /onair/{service_id} | Gets an on-air program of a specified service.
[**get_onair_programs**](OnairApi.md#get_onair_programs) | **GET** /onair | List on-air programs.



## get_onair_program

> Vec<models::WebOnairProgram> get_onair_program(service_id)
Gets an on-air program of a specified service.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**service_id** | **i64** | Mirakurun service ID | [required] |

### Return type

[**Vec<models::WebOnairProgram>**](WebOnairProgram.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_onair_programs

> Vec<models::WebOnairProgram> get_onair_programs()
List on-air programs.

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::WebOnairProgram>**](WebOnairProgram.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

