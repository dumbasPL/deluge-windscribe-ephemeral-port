use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct QBittorrentLoginRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[derive(Serialize, Debug)]
pub struct QBittorrentSetPreferencesRequest<'a> {
    pub json: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QBittorrentPreferences {
    /// set to 0 for random port
    pub listen_port: u64,
    // this is read only, random port is on when listen_port is set to 0
    // pub random_port: bool,
}
