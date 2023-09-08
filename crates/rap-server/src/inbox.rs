use crate::signed::Signed;
use crate::utils::{web_err, WebError};
use axum::http::StatusCode;
use axum::Json;
use log::debug;
use serde_json::Value;

pub async fn json(_signed: Signed, Json(body): Json<Value>) -> Result<Json<Value>, WebError> {
    debug!(
        "Received activity: {}",
        serde_json::to_string(&body).unwrap()
    );

    // TODO: json-ld flatten

    // TODO: store data

    // TODO: queue up any actions

    // let date = chrono::Utc::now().to_rfc2822();
    Err(web_err(StatusCode::NOT_IMPLEMENTED, "Not implemented"))
}
