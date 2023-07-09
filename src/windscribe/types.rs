use std::fmt::Debug;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct WindscribeCsrfToken {
    pub csrf_token: String,
    pub csrf_time: u64,
}

#[derive(Debug, Serialize)]
pub struct WindscribeLoginRequest<'a> {
    pub login: u32,
    pub upgrade: u32,
    pub csrf_token: &'a str,
    pub csrf_time: u64,
    pub username: &'a str,
    pub password: &'a str,
    pub code: &'a str,
}

#[derive(PartialEq)]
pub enum WindscribeEpfInfo {
    Disabled,
    Enabled {
        expires: DateTime<Utc>,
        internal_port: u64,
        external_port: u64,
    },
}

impl Debug for WindscribeEpfInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindscribeEpfInfo::Disabled => write!(f, "Disabled"),
            WindscribeEpfInfo::Enabled {
                expires,
                internal_port,
                external_port,
            } => write!(
                f,
                "Enabled (created: {}, internal port: {}, external port: {})",
                expires, internal_port, external_port
            ),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WindscribeDeleteEpfRequest<'a> {
    pub ctime: u64,
    pub ctoken: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct WindscribeDeleteEpfResponse {
    pub success: u32,
    pub epf: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WindscribeRequestEpfRequest<'a> {
    pub ctime: u64,
    pub ctoken: &'a str,
    pub port: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct WindscribeEpfRequestInfo {
    pub ext: u64,
    pub int: u64,
    pub start_ts: i64,
}

#[derive(Debug, Deserialize)]
pub struct WindscribeRequestEpfResponse {
    pub success: u32,
    pub epf: Option<WindscribeEpfRequestInfo>,
    pub message: Option<String>,
}
