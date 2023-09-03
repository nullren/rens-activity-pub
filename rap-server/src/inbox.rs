use crate::config::Config;
use crate::key;
use crate::signature::Signature;
use crate::users::{PeopleStore, PersonId};
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn json(
    headers: HeaderMap,
    Path(actor): Path<PersonId>,
    Extension(people): Extension<Arc<dyn PeopleStore>>,
    Extension(_cfg): Extension<Config>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // get signature from header
    let signature = header_str(&headers, "signature")?;
    let signature = Signature::from_headers(signature).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Error parsing signature: {}", e),
        )
    })?;

    // TODO: get key for user by signature.key_id
    let person = people.get_or_create(&actor).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error getting person: {}", e),
        )
    })?;

    verify_signature(&headers, signature, person.key)?;

    // let date = chrono::Utc::now().to_rfc2822();
    Ok(Json(json!("OK")))
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Result<&'a str, (StatusCode, String)> {
    headers
        .get(name)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("No header {}", name)))
        .and_then(|h| {
            h.to_str()
                .map_err(|_| (StatusCode::BAD_REQUEST, format!("Invalid header {}", name)))
        })
}

fn verify_signature(
    _headers: &HeaderMap,
    _signature: Signature,
    _key: key::Key,
) -> Result<(), (StatusCode, String)> {
    Ok(())
}
