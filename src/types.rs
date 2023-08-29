use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Actor {
    id: String,
    #[serde(rename = "type")]
    object_type: String,
    inbox: String,
    outbox: String,
    following: String,
    followers: String,
    liked: Option<String>,
    streams: Option<String>,
    #[serde(rename = "preferredUsername")]
    preferred_username: Option<String>,
    name: Option<String>,
    summary: Option<String>,
    url: Option<String>,
    endpoints: HashMap<String, String>,
    icon: Option<Media>,
    image: Option<Media>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Media {
    #[serde(rename = "type")]
    object_type: String,
    #[serde(rename = "mediaType")]
    media_type: String,
    url: String,
}

