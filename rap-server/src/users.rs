use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use std::error::Error;

use crate::key;
use serde::{Deserialize, Serialize};

type PersonId = String;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Person {
    id: String,
    key: key::Key,
}

impl Person {
    pub fn new(id: PersonId) -> Result<Self, Box<dyn Error>> {
        let id = format!("https://ap.rens.page/users/{}", id);
        Ok(Self {
            id: id.clone(),
            key: key::Key::new(id.clone())?,
        })
    }
}

pub async fn json(Path(actor): Path<PersonId>) -> Result<Json<Value>, (StatusCode, String)> {
    let person = Person::new(actor).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error creating person: {}", e),
        )
    })?;
    Ok(Json(json!({
        "@context": [
            "https://www.w3.org/ns/activitystreams",
            "https://w3id.org/security/v1"
        ],
        "id": person.id,
        "type": "Person",
        "inbox": format!("{}/inbox", person.id),
        "publicKey": person.key.public_key().map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error getting public key: {}", e),
            )
        })?,
    })))
}
