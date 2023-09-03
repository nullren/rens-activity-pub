use axum::extract::Query;
use axum::Json;
use serde_json::{json, Value};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Webfinger {
    resource: String,
}

pub async fn json(_webfinger: Query<Webfinger>) -> Json<Value> {
    Json(json!({
      "subject": "acct:ren@ap.rens.page",
      "aliases": [
        "https://ap.rens.page/@Ren",
        "https://ap.rens.page/users/Ren"
      ],
      "links": [
        {
          "rel": "self",
          "type": "application/activity+json",
          "href": "https://ap.rens.page/users/Ren"
        }
      ]
    }))
}
