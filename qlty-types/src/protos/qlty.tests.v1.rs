// @generated
// This file is @generated by prost-build.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CoverageMetadata {
    #[prost(string, tag="1")]
    pub upload_id: ::prost::alloc::string::String,
    #[prost(string, optional, tag="2")]
    pub project_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, tag="29")]
    pub build_id: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub ci: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub ci_url: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub repository_web_url: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub repository_origin_url: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub branch: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub workflow: ::prost::alloc::string::String,
    #[prost(string, tag="9")]
    pub job: ::prost::alloc::string::String,
    #[prost(string, tag="10")]
    pub run: ::prost::alloc::string::String,
    #[prost(string, tag="11")]
    pub run_url: ::prost::alloc::string::String,
    #[prost(string, tag="12")]
    pub commit_sha: ::prost::alloc::string::String,
    #[prost(string, tag="13")]
    pub commit_headline: ::prost::alloc::string::String,
    #[prost(string, tag="14")]
    pub commit_message: ::prost::alloc::string::String,
    #[prost(string, tag="15")]
    pub author_name: ::prost::alloc::string::String,
    #[prost(string, tag="16")]
    pub author_email: ::prost::alloc::string::String,
    #[prost(message, optional, tag="17")]
    pub author_time: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, tag="18")]
    pub committer_name: ::prost::alloc::string::String,
    #[prost(string, tag="19")]
    pub committer_email: ::prost::alloc::string::String,
    #[prost(message, optional, tag="20")]
    pub commit_time: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, tag="21")]
    pub pull_request_number: ::prost::alloc::string::String,
    #[prost(string, tag="22")]
    pub pull_request_url: ::prost::alloc::string::String,
    #[prost(string, tag="23")]
    pub head_ref: ::prost::alloc::string::String,
    #[prost(string, tag="24")]
    pub head_commit: ::prost::alloc::string::String,
    #[prost(string, tag="25")]
    pub base_ref: ::prost::alloc::string::String,
    #[prost(string, tag="26")]
    pub base_commit: ::prost::alloc::string::String,
    #[prost(string, optional, tag="27")]
    pub tag: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, tag="28")]
    pub description: ::prost::alloc::string::String,
    #[prost(message, optional, tag="30")]
    pub uploaded_at: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, tag="31")]
    pub cli_version: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportFile {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="12")]
    pub build_id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub tool: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub format: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub path: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub language: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub contents_md5: ::prost::alloc::string::String,
    #[prost(int64, tag="7")]
    pub size: i64,
    #[prost(string, optional, tag="8")]
    pub project_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="9")]
    pub tag: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="10")]
    pub commit_sha: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag="11")]
    pub uploaded_at: ::core::option::Option<::pbjson_types::Timestamp>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FileCoverage {
    #[prost(string, tag="15")]
    pub upload_id: ::prost::alloc::string::String,
    #[prost(string, tag="12")]
    pub build_id: ::prost::alloc::string::String,
    #[prost(string, tag="1")]
    pub report_id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub path: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub blob_oid: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub contents_md5: ::prost::alloc::string::String,
    #[prost(message, optional, tag="5")]
    pub summary: ::core::option::Option<CoverageSummary>,
    #[prost(int64, repeated, tag="6")]
    pub hits: ::prost::alloc::vec::Vec<i64>,
    #[prost(string, optional, tag="7")]
    pub project_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="8")]
    pub tag: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag="9")]
    pub commit_sha: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag="10")]
    pub uploaded_at: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, tag="13")]
    pub branch: ::prost::alloc::string::String,
    #[prost(string, optional, tag="14")]
    pub pull_request_number: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CoverageSummary {
    #[prost(int64, tag="1")]
    pub covered: i64,
    #[prost(int64, tag="2")]
    pub missed: i64,
    #[prost(int64, tag="3")]
    pub omit: i64,
    #[prost(int64, tag="4")]
    pub total: i64,
}
include!("qlty.tests.v1.serde.rs");
// @@protoc_insertion_point(module)