# \ProgramsStreamApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**check_program_stream**](ProgramsStreamApi.md#check_program_stream) | **HEAD** /programs/{id}/stream | 
[**get_program_stream**](ProgramsStreamApi.md#get_program_stream) | **GET** /programs/{id}/stream | Gets a media stream of a program.



## check_program_stream

> check_program_stream(id, x_mirakurun_priority, decode, pre_filters, post_filters)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | Mirakurun program ID | [required] |
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


## get_program_stream

> get_program_stream(id, x_mirakurun_priority, decode, pre_filters, post_filters)
Gets a media stream of a program.

### A special hack for EPGStation  If the User-Agent header string starts with \"EPGStation/\", this endpoint creates a temporal on-air program tracker if there is no tracker defined in config.yml, which can be reused for tracking changes of the TV program metadata.  The temporal on-air program tracker will be stopped within 1 minute after the streaming stopped.  The metadata will be returned from [/programs/{id}](#/programs/getProgram).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | Mirakurun program ID | [required] |
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

