service_sdk::macros::use_my_sb_entity_protobuf_model!();
#[derive(Clone, PartialEq, ::prost::Message)]
#[my_sb_entity_protobuf_model(topic_id = "trade-log")]
pub struct TradeLogSbModel {
    #[prost(int64, tag = "1")]
    pub date_time_unix_micros: i64,
    #[prost(string, tag = "2")]
    pub trader_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub account_id: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub process_id: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub operation_id: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub message: ::prost::alloc::string::String,
    #[prost(string, tag = "7")]
    pub data: ::prost::alloc::string::String,
}
