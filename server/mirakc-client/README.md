# Rust API client for mirakc-client

No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)


## Overview

This API client was generated by the [OpenAPI Generator](https://openapi-generator.tech) project.  By using the [openapi-spec](https://openapis.org) from a remote server, you can easily generate an API client.

- API version: 4.0.0-dev.0
- Package version: 4.0.0-dev.0
- Generator version: 7.12.0
- Build package: `org.openapitools.codegen.languages.RustClientCodegen`

## Installation

Put the package under your project folder in a directory named `mirakc-client` and add the following to `Cargo.toml` under `[dependencies]`:

```
mirakc-client = { path = "./mirakc-client" }
```

## Documentation for API Endpoints

All URIs are relative to */api*

Class | Method | HTTP request | Description
------------ | ------------- | ------------- | -------------
*ChannelsApi* | [**get_channels**](docs/ChannelsApi.md#get_channels) | **GET** /channels | Lists channels.
*ChannelsServicesStreamApi* | [**get_service_stream_by_channel**](docs/ChannelsServicesStreamApi.md#get_service_stream_by_channel) | **GET** /channels/{type}/{channel}/services/{sid}/stream | Gets a media stream of a service.
*ChannelsServicesStreamApi* | [**head**](docs/ChannelsServicesStreamApi.md#head) | **HEAD** /channels/{type}/{channel}/services/{sid}/stream | 
*ChannelsStreamApi* | [**check_channel_stream**](docs/ChannelsStreamApi.md#check_channel_stream) | **HEAD** /channels/{type}/{channel}/stream | 
*ChannelsStreamApi* | [**get_channel_stream**](docs/ChannelsStreamApi.md#get_channel_stream) | **GET** /channels/{type}/{channel}/stream | Gets a media stream of a channel.
*IptvApi* | [**epg**](docs/IptvApi.md#epg) | **GET** /iptv/epg | Gets an XMLTV document containing all TV program information.
*IptvApi* | [**playlist**](docs/IptvApi.md#playlist) | **GET** /iptv/playlist | Get an M3U8 playlist containing all available services.
*IptvApi* | [**xmltv**](docs/IptvApi.md#xmltv) | **GET** /iptv/xmltv | Gets an XMLTV document containing all TV program information.
*OnairApi* | [**get_onair_program**](docs/OnairApi.md#get_onair_program) | **GET** /onair/{service_id} | Gets an on-air program of a specified service.
*OnairApi* | [**get_onair_programs**](docs/OnairApi.md#get_onair_programs) | **GET** /onair | List on-air programs.
*ProgramsApi* | [**get_program**](docs/ProgramsApi.md#get_program) | **GET** /programs/{id} | Gets a TV program.
*ProgramsApi* | [**get_programs**](docs/ProgramsApi.md#get_programs) | **GET** /programs | Lists TV programs.
*ProgramsStreamApi* | [**check_program_stream**](docs/ProgramsStreamApi.md#check_program_stream) | **HEAD** /programs/{id}/stream | 
*ProgramsStreamApi* | [**get_program_stream**](docs/ProgramsStreamApi.md#get_program_stream) | **GET** /programs/{id}/stream | Gets a media stream of a program.
*RecordingRecordersApi* | [**get_recorder**](docs/RecordingRecordersApi.md#get_recorder) | **GET** /recording/recorders/{program_id} | Gets a recorder.
*RecordingRecordersApi* | [**get_recorders**](docs/RecordingRecordersApi.md#get_recorders) | **GET** /recording/recorders | Lists recorders.
*RecordingRecordersApi* | [**start_recording**](docs/RecordingRecordersApi.md#start_recording) | **POST** /recording/recorders | Starts recording immediately.
*RecordingRecordersApi* | [**stop_recording**](docs/RecordingRecordersApi.md#stop_recording) | **DELETE** /recording/recorders/{program_id} | Stops recording.
*RecordingRecordsApi* | [**get_record**](docs/RecordingRecordsApi.md#get_record) | **GET** /recording/records/{id} | Gets metadata of a record.
*RecordingRecordsApi* | [**get_records**](docs/RecordingRecordsApi.md#get_records) | **GET** /recording/records | Lists records.
*RecordingRecordsApi* | [**remove_record**](docs/RecordingRecordsApi.md#remove_record) | **DELETE** /recording/records/{id} | Removes a record.
*RecordingRecordsStreamApi* | [**check_record_stream**](docs/RecordingRecordsStreamApi.md#check_record_stream) | **HEAD** /recording/records/{id}/stream | 
*RecordingRecordsStreamApi* | [**get_record_stream**](docs/RecordingRecordsStreamApi.md#get_record_stream) | **GET** /recording/records/{id}/stream | Gets a media stream of the content of a record.
*RecordingSchedulesApi* | [**create_recording_schedule**](docs/RecordingSchedulesApi.md#create_recording_schedule) | **POST** /recording/schedules | Books a recording schedule.
*RecordingSchedulesApi* | [**delete_recording_schedule**](docs/RecordingSchedulesApi.md#delete_recording_schedule) | **DELETE** /recording/schedules/{program_id} | Deletes a recording schedule.
*RecordingSchedulesApi* | [**delete_recording_schedules**](docs/RecordingSchedulesApi.md#delete_recording_schedules) | **DELETE** /recording/schedules | Clears recording schedules.
*RecordingSchedulesApi* | [**get_recording_schedule**](docs/RecordingSchedulesApi.md#get_recording_schedule) | **GET** /recording/schedules/{program_id} | Gets a recording schedule.
*RecordingSchedulesApi* | [**get_recording_schedules**](docs/RecordingSchedulesApi.md#get_recording_schedules) | **GET** /recording/schedules | Lists recording schedules.
*ServicesApi* | [**get_logo_image**](docs/ServicesApi.md#get_logo_image) | **GET** /services/{id}/logo | Gets a logo image of a service.
*ServicesApi* | [**get_programs_of_service**](docs/ServicesApi.md#get_programs_of_service) | **GET** /services/{id}/programs | Lists TV programs of a service.
*ServicesApi* | [**get_service**](docs/ServicesApi.md#get_service) | **GET** /services/{id} | Gets a service.
*ServicesApi* | [**get_services**](docs/ServicesApi.md#get_services) | **GET** /services | Lists services.
*ServicesStreamApi* | [**check_service_stream**](docs/ServicesStreamApi.md#check_service_stream) | **HEAD** /services/{id}/stream | 
*ServicesStreamApi* | [**get_service_stream**](docs/ServicesStreamApi.md#get_service_stream) | **GET** /services/{id}/stream | Gets a media stream of a service.
*StatusApi* | [**get_status**](docs/StatusApi.md#get_status) | **GET** /status | Gets current status information.
*StreamApi* | [**check_channel_stream**](docs/StreamApi.md#check_channel_stream) | **HEAD** /channels/{type}/{channel}/stream | 
*StreamApi* | [**check_program_stream**](docs/StreamApi.md#check_program_stream) | **HEAD** /programs/{id}/stream | 
*StreamApi* | [**check_record_stream**](docs/StreamApi.md#check_record_stream) | **HEAD** /recording/records/{id}/stream | 
*StreamApi* | [**check_service_stream**](docs/StreamApi.md#check_service_stream) | **HEAD** /services/{id}/stream | 
*StreamApi* | [**get_channel_stream**](docs/StreamApi.md#get_channel_stream) | **GET** /channels/{type}/{channel}/stream | Gets a media stream of a channel.
*StreamApi* | [**get_program_stream**](docs/StreamApi.md#get_program_stream) | **GET** /programs/{id}/stream | Gets a media stream of a program.
*StreamApi* | [**get_record_stream**](docs/StreamApi.md#get_record_stream) | **GET** /recording/records/{id}/stream | Gets a media stream of the content of a record.
*StreamApi* | [**get_service_stream**](docs/StreamApi.md#get_service_stream) | **GET** /services/{id}/stream | Gets a media stream of a service.
*StreamApi* | [**get_service_stream_by_channel**](docs/StreamApi.md#get_service_stream_by_channel) | **GET** /channels/{type}/{channel}/services/{sid}/stream | Gets a media stream of a service.
*StreamApi* | [**head**](docs/StreamApi.md#head) | **HEAD** /channels/{type}/{channel}/services/{sid}/stream | 
*TunersApi* | [**get_tuner**](docs/TunersApi.md#get_tuner) | **GET** /tuners/{index} | Gets a tuner model.
*TunersApi* | [**get_tuners**](docs/TunersApi.md#get_tuners) | **GET** /tuners | Lists tuners enabled in `config.yml`.
*VersionApi* | [**check_version**](docs/VersionApi.md#check_version) | **GET** /version | Gets version information.


## Documentation For Models

 - [ChannelType](docs/ChannelType.md)
 - [MirakurunChannel](docs/MirakurunChannel.md)
 - [MirakurunChannelServicesInner](docs/MirakurunChannelServicesInner.md)
 - [MirakurunProgram](docs/MirakurunProgram.md)
 - [MirakurunProgramAudio](docs/MirakurunProgramAudio.md)
 - [MirakurunProgramAudiosInner](docs/MirakurunProgramAudiosInner.md)
 - [MirakurunProgramGenresInner](docs/MirakurunProgramGenresInner.md)
 - [MirakurunProgramRelatedItemsInner](docs/MirakurunProgramRelatedItemsInner.md)
 - [MirakurunProgramSeries](docs/MirakurunProgramSeries.md)
 - [MirakurunProgramVideo](docs/MirakurunProgramVideo.md)
 - [MirakurunService](docs/MirakurunService.md)
 - [MirakurunServiceChannel](docs/MirakurunServiceChannel.md)
 - [MirakurunTuner](docs/MirakurunTuner.md)
 - [MirakurunTunerUsersInner](docs/MirakurunTunerUsersInner.md)
 - [RecordingFailedReason](docs/RecordingFailedReason.md)
 - [RecordingFailedReasonOneOf](docs/RecordingFailedReasonOneOf.md)
 - [RecordingFailedReasonOneOf1](docs/RecordingFailedReasonOneOf1.md)
 - [RecordingFailedReasonOneOf2](docs/RecordingFailedReasonOneOf2.md)
 - [RecordingFailedReasonOneOf3](docs/RecordingFailedReasonOneOf3.md)
 - [RecordingFailedReasonOneOf4](docs/RecordingFailedReasonOneOf4.md)
 - [RecordingFailedReasonOneOf5](docs/RecordingFailedReasonOneOf5.md)
 - [RecordingOptions](docs/RecordingOptions.md)
 - [RecordingScheduleState](docs/RecordingScheduleState.md)
 - [Version](docs/Version.md)
 - [WebContentInfo](docs/WebContentInfo.md)
 - [WebOnairProgram](docs/WebOnairProgram.md)
 - [WebProcessModel](docs/WebProcessModel.md)
 - [WebRecord](docs/WebRecord.md)
 - [WebRecordingInfo](docs/WebRecordingInfo.md)
 - [WebRecordingRecorder](docs/WebRecordingRecorder.md)
 - [WebRecordingSchedule](docs/WebRecordingSchedule.md)
 - [WebRecordingScheduleInput](docs/WebRecordingScheduleInput.md)
 - [WebRecordingStatus](docs/WebRecordingStatus.md)
 - [WebTimeshiftRecord](docs/WebTimeshiftRecord.md)
 - [WebTimeshiftRecorder](docs/WebTimeshiftRecorder.md)


To get access to the crate's generated documentation, use:

```
cargo doc --open
```

## Author



