use axum::extract::Path;
use axum::{Extension, Json};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::key;
use serde::{Deserialize, Serialize};

type PersonId = String;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Person {
    id: String,
    key: key::Key,
}

impl Person {
    pub fn new(id: String) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            id,
            key: key::Key::new(id.clone())?,
        })
    }
}

#[derive(Clone)]
pub struct PeopleStore {
    lookup: Arc<Mutex<HashMap<PersonId, Person>>>,
}

impl PeopleStore {
    pub fn new() -> Self {
        Self {
            lookup: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // TODO: This should be in a database
    pub async fn add(&self, person: Person) -> Result<(), Box<dyn Error + '_>> {
        let mut lookup = self.lookup.lock()?;
        lookup.insert(person.id.clone(), person);
        Ok(())
    }

    // TODO: This should be in a database
    pub async fn get(&self, id: &PersonId) -> Result<Option<&Person>, Box<dyn Error + '_>> {
        let lookup = self.lookup.lock()?;
        Ok(lookup.get(id))
    }

    pub async fn get_or_create(&self, id: &PersonId) -> Result<&Person, Box<dyn Error + '_>> {
        let mut lookup = self.lookup.lock()?;
        if let Some(person) = lookup.get(id) {
            return Ok(person);
        }
        let person = Person::new(id.clone())?;
        lookup.insert(id.clone(), person.clone());
        Ok(lookup.get(id).unwrap())
    }
}

pub async fn json(
    actor: Path<PersonId>,
    Extension(peopleStore): Extension<PeopleStore>,
) -> Json<Value> {
    let person = peopleStore.get_or_create(&actor).await.unwrap();
    return Json(json!({
        "@context": [
            "https://www.w3.org/ns/activitystreams",
            "https://w3id.org/security/v1"
        ],

        "id": person.id,
        "type": "Person",
        "inbox": format!("{}/inbox", person.id),

        "publicKey": person.key.public_key().unwrap(),
    }));
}
