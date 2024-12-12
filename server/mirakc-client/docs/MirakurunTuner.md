# MirakurunTuner

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**command** | Option<**String**> | A command to use getting a media stream from the tuner. | [optional]
**index** | **i32** | The index of the tuner defined in `config.yml`. | 
**is_available** | **bool** | Always `true`. | 
**is_fault** | **bool** | Always `false`. | 
**is_free** | **bool** | `true` if the tuner is free, `false` otherwise. | 
**is_remote** | **bool** | Always `false`. | 
**is_using** | **bool** | `false` if the tuner is free, `true` otherwise. | 
**name** | **String** | The name of the tuner defined in `config.yml`. | 
**pid** | Option<**i32**> | PID of a process to run the command. | [optional]
**types** | [**Vec<models::ChannelType>**](ChannelType.md) | Channel types supported by the tuner. | 
**users** | [**Vec<models::MirakurunTunerUsersInner>**](MirakurunTuner_users_inner.md) | Users of the tuner. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


