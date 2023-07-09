use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Debug)]
pub struct TransmissionRequest<'a> {
    pub method: &'a str,
    pub arguments: Value,
}

#[derive(Deserialize, Debug)]
pub struct TransmissionResponse {
    pub arguments: Value,
    pub result: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransmissionSessionArgumentsPort {
    #[serde(rename = "peer-port")]
    pub peer_port: u64,
    #[serde(rename = "peer-port-random-on-start")]
    pub peer_port_random_on_start: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransmissionSessionArgumentsVersion {
    #[serde(rename = "version")]
    pub version: String,
}
