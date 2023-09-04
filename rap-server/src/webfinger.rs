use axum::extract::Query;
use axum::http::StatusCode;
use axum::{Extension, Json};
use log::warn;
use serde_json::{json, Value};

use crate::Config;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Webfinger {
    resource: String,
}

pub async fn json(
    webfinger: Query<Webfinger>,
    Extension(cfg): Extension<Config>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let domain = cfg.domain.clone();
    let resource = webfinger.resource.clone().to_lowercase();

    let error = || {
        warn!("Invalid webfinger resource");
        (StatusCode::BAD_REQUEST, "Invalid resource".to_string());
    };

    let id = resource
        .strip_prefix("acct:")
        .ok_or_else(error)?
        .strip_suffix(&domain)
        .ok_or_else(error)?
        .strip_suffix("@")
        .ok_or_else(error)?;

    // TODO: check if id exists

    Ok(Json(json!({
      "subject": format!("acct:{}@{}", id, domain),
      "aliases": [
        format!("https://{}/@{}", domain, id),
        format!("https://{}/users/{}", domain, id)
      ],
      "links": [
        {
          "rel": "self",
          "type": "application/activity+json",
          "href": format!("https://{}/users/{}", domain, id)
        }
      ]
    })))
}
