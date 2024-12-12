# \ServicesApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_logo_image**](ServicesApi.md#get_logo_image) | **GET** /services/{id}/logo | Gets a logo image of a service.
[**get_programs_of_service**](ServicesApi.md#get_programs_of_service) | **GET** /services/{id}/programs | Lists TV programs of a service.
[**get_service**](ServicesApi.md#get_service) | **GET** /services/{id} | Gets a service.
[**get_services**](ServicesApi.md#get_services) | **GET** /services | Lists services.



## get_logo_image

> get_logo_image(id)
Gets a logo image of a service.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | Mirakurun service ID | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: image/png

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_programs_of_service

> Vec<models::MirakurunProgram> get_programs_of_service(id)
Lists TV programs of a service.

The list contains TV programs that have ended.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | Mirakurun service ID | [required] |

### Return type

[**Vec<models::MirakurunProgram>**](MirakurunProgram.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_service

> models::MirakurunService get_service(id)
Gets a service.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | Mirakurun service ID | [required] |

### Return type

[**models::MirakurunService**](MirakurunService.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_services

> Vec<models::MirakurunService> get_services()
Lists services.

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::MirakurunService>**](MirakurunService.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

