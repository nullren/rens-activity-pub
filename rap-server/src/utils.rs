use axum::http::StatusCode;
use base64::engine::general_purpose;
use base64::Engine;
use log::warn;

pub fn base64_decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let decoded = general_purpose::STANDARD.decode(data)?;
    Ok(decoded)
}

pub type WebError = (StatusCode, String);
pub fn web_err<S: Into<String>>(status: StatusCode, msg: S) -> WebError {
    let msg = msg.into();
    warn!("{}", msg);
    (status, msg)
}

pub fn web_err_500<S: Into<String>>(msg: S) -> WebError {
    web_err(StatusCode::INTERNAL_SERVER_ERROR, msg)
}

pub fn web_err_400<S: Into<String>>(msg: S) -> WebError {
    web_err(StatusCode::BAD_REQUEST, msg)
}
