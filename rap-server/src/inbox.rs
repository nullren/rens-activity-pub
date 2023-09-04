use crate::config::Config;
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

    let _comparison = rebuild_sig_str(&actor, &headers, &signature);

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

fn rebuild_sig_str(
    account: &PersonId,
    headers: &HeaderMap,
    signature: &Signature,
) -> String {
    signature
        .headers
        .iter()
        .map(|header| {
            if header == "(request-target)" {
                format!("(request-target): post /users/{}/inbox", account)
            } else {
                let header = header.to_lowercase();
                let value = header_str(headers, &header).unwrap_or("");
                format!("{}: {}", header, value)
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;
    use super::*;

    #[test]
    fn test_rebuild_sig_str() {
        // Create a mock HeaderMap
        let mut headers = HeaderMap::new();
        headers.insert("Host", HeaderValue::from_static("example.com"));
        headers.insert("Date", HeaderValue::from_static("Sun, 06 Nov 2021 08:49:37 GMT"));

        // Create a mock Signature
        let signature = Signature {
            key_id: "".to_string(),
            headers: vec![
                String::from("(request-target)"),
                String::from("Host"),
                String::from("Date"),
            ],
            // Add other fields if the Signature struct has more
            signature: "".to_string(),
        };

        // Create a mock PersonId
        let person_id = "alice".to_string();

        // Call the function
        let result = rebuild_sig_str(&person_id, &headers, &signature);

        // Expected signature string
        let expected_result = "(request-target): post /users/alice/inbox\nhost: example.com\ndate: Sun, 06 Nov 2021 08:49:37 GMT";

        assert_eq!(result, expected_result, "Rebuilt signature string did not match expected string");
    }

}
