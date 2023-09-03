use crate::config::Config;
use crate::signature::Signature;
use crate::users::PersonId;
use axum::extract::Path;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::Extension;
use hyper::Body;

pub async fn json(
    headers: HeaderMap,
    req: Request<Body>,
    Path(actor): Path<PersonId>,
    Extension(cfg): Extension<Config>,
) -> Result<String, (StatusCode, String)> {
    // get signature from header
    let signature = header_str(&headers, "signature")?;
    let signature = Signature::from_headers(signature).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Error parsing signature: {}", e),
        )
    })?;

    // TODO: get key for user by signature.key_id

    verify_signature(&headers, signature)?;

    let date = chrono::Utc::now().to_rfc2822();
    Ok("OK".to_string())
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
) -> Result<(), (StatusCode, String)> {
    Ok(())
}
