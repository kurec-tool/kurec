# \ProgramsApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_program**](ProgramsApi.md#get_program) | **GET** /programs/{id} | Gets a TV program.
[**get_programs**](ProgramsApi.md#get_programs) | **GET** /programs | Lists TV programs.



## get_program

> Vec<models::MirakurunProgram> get_program(id)
Gets a TV program.

### A special hack for EPGStation  If the User-Agent header string starts with \"EPGStation/\", this endpoint returns information contained in EIT[p/f] if it exists. Otherwise, information contained in EIT[schedule] is returned.  EPGStation calls this endpoint in order to update the start time and the duration of the TV program while recording.  The intention of this call is assumed that EPGStation wants to get the TV program information equivalent to EIT[p].  However, this endpoint should return information contained in EIT[schedule] basically in a web API consistency point of view.  Information contained in EIT[p/f] should be returned from other endpoints.  See also [/programs/{id}/stream](#/stream/getProgramStream).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | Mirakurun program ID | [required] |

### Return type

[**Vec<models::MirakurunProgram>**](MirakurunProgram.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_programs

> Vec<models::MirakurunProgram> get_programs()
Lists TV programs.

The list contains TV programs that have ended.  A newer Mirakurun returns information contained in EIT[schedule] overridded by EIT[p/f] from this endpoint.  This may cause

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::MirakurunProgram>**](MirakurunProgram.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

