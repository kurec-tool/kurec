# \TunersApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_tuner**](TunersApi.md#get_tuner) | **GET** /tuners/{index} | Gets a tuner model.
[**get_tuners**](TunersApi.md#get_tuners) | **GET** /tuners | Lists tuners enabled in `config.yml`.



## get_tuner

> models::MirakurunTuner get_tuner(index)
Gets a tuner model.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**index** | **i32** | Tuner index | [required] |

### Return type

[**models::MirakurunTuner**](MirakurunTuner.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tuners

> Vec<models::MirakurunTuner> get_tuners()
Lists tuners enabled in `config.yml`.

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::MirakurunTuner>**](MirakurunTuner.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

