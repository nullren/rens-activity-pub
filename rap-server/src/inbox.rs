use crate::key::PublicKey;
use crate::signature::Signature;
use crate::users::PersonId;
use crate::utils::base64_decode;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use log::warn;
use serde_json::Value;

pub async fn json(
    headers: HeaderMap,
    Path(actor): Path<PersonId>,
) -> Result<Json<Value>, (StatusCode, String)> {
    verify_headers(&headers, &actor).await?;

    // let date = chrono::Utc::now().to_rfc2822();
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented".to_string()))
}
fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Result<&'a str, (StatusCode, String)> {
    headers
        .get(name)
        .ok_or_else(|| {
            warn!("No header {}", name);
            (StatusCode::BAD_REQUEST, format!("No header {}", name))
        })
        .and_then(|h| {
            h.to_str().map_err(|_| {
                warn!("Invalid header {}", name);
                (StatusCode::BAD_REQUEST, format!("Invalid header {}", name))
            })
        })
}

fn rebuild_sig_str(account: &PersonId, headers: &HeaderMap, signature: &Signature) -> String {
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

async fn verify_headers(headers: &HeaderMap, actor: &PersonId) -> Result<(), (StatusCode, String)> {
    let signature = header_str(&headers, "signature")?;
    let signature = Signature::from_headers(signature).map_err(|e| {
        warn!("Error parsing signature: {}", e);
        (
            StatusCode::BAD_REQUEST,
            format!("Error parsing signature: {}", e),
        )
    })?;

    let decoded_signature = base64_decode(&signature.signature).map_err(|e| {
        warn!("Error decoding signature: {}", e);
        (
            StatusCode::BAD_REQUEST,
            format!("Error decoding signature: {}", e),
        )
    })?;

    let comparison = rebuild_sig_str(&actor, &headers, &signature);

    let pubkey = PublicKey::from_remote(&signature.key_id)
        .await
        .map_err(|e| {
            warn!("Error loading public key: {}", e);
            (
                StatusCode::BAD_REQUEST,
                format!("Error loading public key: {}", e),
            )
        })?;

    println!("pubkey: {}", serde_json::to_string(&pubkey).unwrap());
    println!("comparison: {}", comparison);

    pubkey
        .verify(&decoded_signature, comparison.as_bytes())
        .map_err(|e| {
            warn!("Error verifying signature: {}. {:?}", e, headers);
            (
                StatusCode::BAD_REQUEST,
                format!("Error verifying signature: {}", e),
            )
        })?;

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
