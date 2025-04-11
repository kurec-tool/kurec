# \StreamApi

All URIs are relative to */api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**check_channel_stream**](StreamApi.md#check_channel_stream) | **HEAD** /channels/{type}/{channel}/stream | 
[**check_program_stream**](StreamApi.md#check_program_stream) | **HEAD** /programs/{id}/stream | 
[**check_record_stream**](StreamApi.md#check_record_stream) | **HEAD** /recording/records/{id}/stream | 
[**check_service_stream**](StreamApi.md#check_service_stream) | **HEAD** /services/{id}/stream | 
[**get_channel_stream**](StreamApi.md#get_channel_stream) | **GET** /channels/{type}/{channel}/stream | Gets a media stream of a channel.
[**get_program_stream**](StreamApi.md#get_program_stream) | **GET** /programs/{id}/stream | Gets a media stream of a program.
[**get_record_stream**](StreamApi.md#get_record_stream) | **GET** /recording/records/{id}/stream | Gets a media stream of the content of a record.
[**get_service_stream**](StreamApi.md#get_service_stream) | **GET** /services/{id}/stream | Gets a media stream of a service.
[**get_service_stream_by_channel**](StreamApi.md#get_service_stream_by_channel) | **GET** /channels/{type}/{channel}/services/{sid}/stream | Gets a media stream of a service.
[**head**](StreamApi.md#head) | **HEAD** /channels/{type}/{channel}/services/{sid}/stream | 



## check_channel_stream

> check_channel_stream(r#type, channel, x_mirakurun_priority, decode, pre_filters, post_filters)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**r#type** | [**ChannelType**](.md) | Channel type | [required] |
**channel** | **String** | Channel number | [required] |
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


## check_record_stream

> check_record_stream(id, pre_filters, post_filters)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Record ID | [required] |
**pre_filters** | Option<[**Vec<String>**](String.md)> | pre-filters |  |
**post_filters** | Option<[**Vec<String>**](String.md)> | post-filters |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


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


## get_channel_stream

> get_channel_stream(r#type, channel, x_mirakurun_priority, decode, pre_filters, post_filters)
Gets a media stream of a channel.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**r#type** | [**ChannelType**](.md) | Channel type | [required] |
**channel** | **String** | Channel number | [required] |
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


## get_record_stream

> get_record_stream(id, pre_filters, post_filters)
Gets a media stream of the content of a record.

It's possible to get a media stream of the record even while it's recording.  In this case, data will be sent when data is appended to the content file event if the stream reaches EOF at that point.  The streaming will stop within 2 seconds after the stream reaches the *true* EOF.  A request for a record without content file always returns status code 204.  A range request with filters always causes an error response with status code 400.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Record ID | [required] |
**pre_filters** | Option<[**Vec<String>**](String.md)> | pre-filters |  |
**post_filters** | Option<[**Vec<String>**](String.md)> | post-filters |  |

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


## get_service_stream_by_channel

> get_service_stream_by_channel(r#type, channel, sid, x_mirakurun_priority, decode, pre_filters, post_filters)
Gets a media stream of a service.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**r#type** | [**ChannelType**](.md) | Channel type | [required] |
**channel** | **String** | Channel number | [required] |
**sid** | **i32** | Service ID (not Mirakurun Service ID) | [required] |
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


## head

> head(r#type, channel, sid, x_mirakurun_priority, decode, pre_filters, post_filters)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**r#type** | [**ChannelType**](.md) | Channel type | [required] |
**channel** | **String** | Channel number | [required] |
**sid** | **i32** | Service ID (not Mirakurun Service ID) | [required] |
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

