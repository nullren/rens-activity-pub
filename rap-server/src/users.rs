use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;

use crate::key;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

pub type PersonId = String;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Person {
    id: String,
    key: key::Key,
}

#[async_trait::async_trait]
trait PeopleStore: Send + Sync {
    async fn get_or_create(&self, id: &PersonId) -> Result<Person, Box<dyn Error>>;
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

struct InMemoryPeopleStore {
    people: Mutex<HashMap<PersonId, Person>>,
}

impl InMemoryPeopleStore {
    fn new() -> Self {
        Self {
            people: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl PeopleStore for InMemoryPeopleStore {
    async fn get_or_create(&self, id: &PersonId) -> Result<Person, Box<dyn Error>> {
        let mut people = self.people.lock().await;

        if !people.contains_key(id) {
            let person = Person::new(id.clone()).unwrap();
            people.insert(id.clone(), person);
        }

        let p = people.get(id).unwrap();
        return Ok(p.clone());
    }
}
