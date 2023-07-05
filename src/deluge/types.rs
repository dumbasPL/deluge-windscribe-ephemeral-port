use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Debug)]
pub struct DelugeRequest {
    pub method: String,
    pub params: Vec<Value>,
    pub id: u32,
}

#[derive(Deserialize, Debug)]
pub struct DelugeError {
    pub code: u32,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct DelugeResponse {
    pub result: Value,
    pub error: Option<DelugeError>,
    pub id: u32,
}

pub struct DelugeHost {
    pub id: String,
    pub ip: String,
    pub port: u64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DelugeConfig {
    pub random_port: bool,
    pub listen_ports: [u64; 2],
}
