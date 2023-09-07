use crate::key::PublicKey;
use crate::signature::Signature;
use crate::users::PersonId;
use crate::utils::{base64_decode, web_err_400, web_err_500, WebError};
use axum::async_trait;
use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;
use axum::http::HeaderMap;
use log::debug;

/// # Signed Extractor
///
/// The `Signed` extractor is responsible for verifying the authenticity of incoming
/// HTTP requests by checking the provided headers, such as the "signature" header,
/// against a specified set of criteria. It ensures that the request is signed and
/// valid before processing further.
///
/// This extractor is typically used in Axum route handlers to authenticate and
/// authorize incoming requests from clients.
///
/// ## Usage
///
/// To use the `Signed` extractor, include it as a parameter in your route handlers.
/// When a request is processed by a handler that includes this extractor, it will
/// attempt to validate the request headers. If the headers are valid, the extractor
/// returns `Ok(Signed)`, indicating that the request is authenticated. If the headers
/// are invalid or missing, it returns an error with an appropriate status code and
/// error message.
///
/// ## Example
///
/// ```rust
/// use axum::prelude::*;
/// use axum::http::StatusCode;
///
/// async fn my_handler(_signed: Signed) -> impl IntoResponse {
///     // Your request handling logic here
///     // The `Signed` parameter ensures that the request is authenticated.
///     // If not, it returns an error response with a status code.
///     // ...
/// }
/// ```
///
/// ## Dependencies
///
/// The `Signed` extractor depends on the following external crates:
///
/// - `axum`: The Axum web framework for routing and handling HTTP requests.
/// - `log`: A logging framework used for logging warning messages.
/// - `crate::signature::Signature`: Your custom signature implementation.
/// - `crate::users::PersonId`: Your custom user identifier type.
/// - `crate::utils::base64_decode`: A utility function for base64 decoding.
///
/// ## Errors
///
/// If the headers are invalid or missing, the `Signed` extractor will return an
/// error with a status code and an error message. The possible status codes include:
///
/// - `StatusCode::BAD_REQUEST`: Indicates that the request headers are invalid or
///   missing required headers.
/// - Other status codes as needed based on your application's requirements.
///
/// ## See Also
///
/// - [`verify_headers`](fn.verify_headers.html): The internal function used to
///   verify request headers.
/// - [`rebuild_sig_str`](fn.rebuild_sig_str.html): The internal function used to
///   rebuild the signature string for verification.
/// - [`header_str`](fn.header_str.html): A utility function used to retrieve header
///   values from a `HeaderMap`.
///
/// [Axum]: https://docs.rs/axum
/// [log]: https://docs.rs/log
/// [`crate::signature::Signature`]: ./struct.Signature.html
/// [`crate::users::PersonId`]: ./struct.PersonId.html
/// [`crate::utils::base64_decode`]: ./fn.base64_decode.html
/// [`verify_headers`]: ./fn.verify_headers.html
/// [`rebuild_sig_str`]: ./fn.rebuild_sig_str.html
/// [`header_str`]: ./fn.header_str.html
pub struct Signed;

#[async_trait]
impl<S> FromRequestParts<S> for Signed
where
    S: Send + Sync,
{
    type Rejection = WebError;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // need to get Path
        use axum::RequestPartsExt;
        let Path(person_id) = parts
            .extract::<Path<PersonId>>()
            .await
            .map_err(|_| web_err_500("Could not extract person_id from path"))?;

        let headers = parts.headers.clone();

        verify_headers(&headers, &person_id).await?;

        Ok(Signed)
    }
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Result<&'a str, WebError> {
    headers
        .get(name)
        .ok_or_else(|| web_err_400(format!("No header {}", name)))
        .and_then(|h| {
            h.to_str()
                .map_err(|_| web_err_400(format!("Invalid header {}", name)))
        })
}

fn rebuild_sig_str(account: &PersonId, headers: &HeaderMap, signature: &Signature) -> String {
    signature
        .headers
        .iter()
        .map(|header| {
            if header == "(request-target)" {
                // we can take this shortcut because we run this endpoint
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

async fn verify_headers(headers: &HeaderMap, actor: &PersonId) -> Result<(), WebError> {
    let signature = header_str(headers, "signature")?;
    let signature = Signature::from_headers(signature)
        .map_err(|e| web_err_400(format!("Error parsing signature: {}", e)))?;

    let decoded_signature = base64_decode(&signature.signature)
        .map_err(|e| web_err_400(format!("Error decoding signature: {}", e)))?;

    let comparison = rebuild_sig_str(actor, headers, &signature);

    let pubkey = PublicKey::from_remote(&signature.key_id)
        .await
        .map_err(|e| web_err_400(format!("Error loading public key: {}", e)))?;

    debug!("pubkey: {}", serde_json::to_string(&pubkey).unwrap());
    debug!("comparison: {}", comparison);

    pubkey
        .verify(comparison.as_bytes(), &decoded_signature)
        .map_err(|e| web_err_400(format!("Error verifying signature: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use std::{assert_eq, vec};

    #[test]
    fn test_rebuild_sig_str() {
        // Create a mock HeaderMap
        let mut headers = HeaderMap::new();
        headers.insert("Host", HeaderValue::from_static("example.com"));
        headers.insert(
            "Date",
            HeaderValue::from_static("Sun, 06 Nov 2021 08:49:37 GMT"),
        );

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

        assert_eq!(
            result, expected_result,
            "Rebuilt signature string did not match expected string"
        );
    }

    #[tokio::test]
    async fn test_verify_headers_from_remote() {
        // Create a mock HeaderMap
        let mut headers = HeaderMap::new();
        // headers from: {"host": "ap.rens.page", "connection": "close", "user-agent": "http.rb/5.1.1 (Mastodon/4.1.6; +https://hotdog.place/)", "date": "Mon, 04 Sep 2023 20:49:38 GMT", "accept-encoding": "gzip", "digest": "SHA-256=x0QZ2hdf3slWOdA4/DyxLEv4uEzU/FgjP9ho8EzR8sk=", "content-type": "application/activity+json", "signature": "keyId=\"https://hotdog.place/users/renning#main-key\",algorithm=\"rsa-sha256\",headers=\"(request-target) host date digest content-type\",signature=\"GAoq49DfHXRwU8N5bwZAVoU3f5fUR5BPaWLVTG/6QlTJB12lRV29KLxN0pMbcHgzKoTWepdPcIPYZXVGR12+VBoSW46bSKVhFZ8thV/I6Sm/Xqmsz46LJNCETODyOvtFYAnagYUBTq5sbBznovWJNaRkM38fQII+oXV3V1Ku9Y10kPXrQL0JwRoNvzrvAzZJBLGKArdBB9yeVgfLAp3NwmZAwawSSBfh73sBqcTgfrZvjN95xvJWfFvveZINV1Fb4EIfFCZJHcNWNLG8d0PEsk5TjFqKuTjkgYWP5xogiepN8BJfPB+QPfdTPlWr+Gos2pDgo83sna5NehHowgkDiA==\"", "x-request-id": "8a10afb4-180b-4599-85a7-d987e92c0086", "x-forwarded-for": "141.95.205.41", "x-forwarded-proto": "https", "x-forwarded-port": "443", "via": "1.1 vegur", "connect-time": "0", "x-request-start": "1693860578252", "total-route-time": "0", "content-length": "222"}
        headers.insert("host", HeaderValue::from_static("ap.rens.page"));
        headers.insert("connection", HeaderValue::from_static("close"));
        headers.insert(
            "user-agent",
            HeaderValue::from_static("http.rb/5.1.1 (Mastodon/4.1.6; +https://hotdog.place/)"),
        );
        headers.insert(
            "date",
            HeaderValue::from_static("Mon, 04 Sep 2023 20:49:38 GMT"),
        );
        headers.insert("accept-encoding", HeaderValue::from_static("gzip"));
        headers.insert(
            "digest",
            HeaderValue::from_static("SHA-256=x0QZ2hdf3slWOdA4/DyxLEv4uEzU/FgjP9ho8EzR8sk="),
        );
        headers.insert(
            "content-type",
            HeaderValue::from_static("application/activity+json"),
        );
        headers.insert(
            "signature",
            HeaderValue::from_static("keyId=\"https://hotdog.place/users/renning#main-key\",algorithm=\"rsa-sha256\",headers=\"(request-target) host date digest content-type\",signature=\"GAoq49DfHXRwU8N5bwZAVoU3f5fUR5BPaWLVTG/6QlTJB12lRV29KLxN0pMbcHgzKoTWepdPcIPYZXVGR12+VBoSW46bSKVhFZ8thV/I6Sm/Xqmsz46LJNCETODyOvtFYAnagYUBTq5sbBznovWJNaRkM38fQII+oXV3V1Ku9Y10kPXrQL0JwRoNvzrvAzZJBLGKArdBB9yeVgfLAp3NwmZAwawSSBfh73sBqcTgfrZvjN95xvJWfFvveZINV1Fb4EIfFCZJHcNWNLG8d0PEsk5TjFqKuTjkgYWP5xogiepN8BJfPB+QPfdTPlWr+Gos2pDgo83sna5NehHowgkDiA==\""),
        );
        headers.insert(
            "x-request-id",
            HeaderValue::from_static("8a10afb4-180b-4599-85a7-d987e92c0086"),
        );
        headers.insert("x-forwarded-for", HeaderValue::from_static(""));
        headers.insert("x-forwarded-proto", HeaderValue::from_static("https"));
        headers.insert("x-forwarded-port", HeaderValue::from_static("443"));
        headers.insert("via", HeaderValue::from_static("1.1 vegur"));
        headers.insert("connect-time", HeaderValue::from_static("0"));
        headers.insert("x-request-start", HeaderValue::from_static("1693860578252"));
        headers.insert("total-route-time", HeaderValue::from_static("0"));
        headers.insert("content-length", HeaderValue::from_static("222"));

        // Create a mock PersonId
        let person_id = "test2".to_string();

        verify_headers(&headers, &person_id).await.unwrap();
    }
}
