use crate::crypto;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Key {
    owner: String,
    #[serde(rename = "privateKey")]
    private_key_pem: String,
    #[serde(rename = "publicKey")]
    public_key_pem: String,
}

impl Key {
    pub fn new(owner: String) -> Result<Self, Box<dyn Error>> {
        let (private_key_pem, public_key_pem) = crypto::generate_keypair()?;
        Ok(Self {
            owner,
            private_key_pem,
            public_key_pem,
        })
    }

    pub fn public_key(&self) -> Result<PublicKey, Box<dyn Error>> {
        Ok(PublicKey {
            id: format!("{}/#main-key", self.owner),
            owner: self.owner.clone(),
            public_key_pem: self.public_key_pem.clone(),
        })
    }

    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        crypto::sign(&self.private_key_pem, data)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PublicKey {
    id: String,
    owner: String,
    #[serde(rename = "publicKeyPem")]
    public_key_pem: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Actor {
    id: String,
    inbox: String,
    #[serde(rename = "publicKey")]
    public_key: PublicKey,
}

impl PublicKey {
    pub async fn from_remote(id: &str) -> Result<Self, Box<dyn Error>> {
        let resp = reqwest::Client::new()
            .get(id)
            .header(
                "Accept",
                "application/ld+json; profile=\"http://www.w3.org/ns/activitystreams\"",
            )
            .send()
            .await?;
        let resp = resp.json::<Actor>().await?;
        Ok(resp.public_key)
    }

    pub fn verify(&self, data: &[u8], sig: &[u8]) -> Result<(), Box<dyn Error>> {
        crypto::verify(&self.public_key_pem, data, sig)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::base64_decode;

    #[test]
    fn test_key_creation() {
        let key = Key::new("owner1".to_string());
        assert!(key.is_ok(), "Key creation failed");
    }

    #[test]
    fn test_public_key_derivation() {
        let key = Key::new("owner2".to_string()).expect("Failed to create key");
        let public_key = key.public_key();
        assert!(public_key.is_ok(), "Public key derivation failed");
    }

    #[test]
    fn test_sign_and_verify() {
        let key = Key::new("owner3".to_string()).expect("Failed to create key");
        let data = b"some data to sign";

        // Sign the data
        let signature = key.sign(data).expect("Failed to sign data");

        // Verify the signature
        let verification_result = key
            .public_key()
            .expect("Failed to make public key")
            .verify(data, &signature);
        assert!(verification_result.is_ok(), "Signature verification failed");
    }

    #[test]
    fn test_sign_and_verify_failure() {
        let key = Key::new("owner4".to_string()).expect("Failed to create key");
        let data = b"some data to sign";
        let wrong_data = b"some other data";

        // Sign the data
        let signature = key.sign(data).expect("Failed to sign data");

        // Try to verify with wrong data
        let verification_result = key
            .public_key()
            .expect("Failed to make public key")
            .verify(wrong_data, &signature);
        assert!(
            verification_result.is_err(),
            "Signature verification should fail for wrong data"
        );
    }

    #[tokio::test]
    async fn test_remote_public_key() {
        let key = PublicKey::from_remote("https://hotdog.place/users/renning#main-key").await;
        assert!(key.is_ok(), "Failed to fetch remote public key");
    }

    #[test]
    fn test_verify_signature() {
        // https://github.com/mastodon/mastodon/blob/f80f426c57d5a5e1d289372ef7c323741d27c768/app/lib/request.rb#L179C1-L196C6
        let signature = "GAoq49DfHXRwU8N5bwZAVoU3f5fUR5BPaWLVTG/6QlTJB12lRV29KLxN0pMbcHgzKoTWepdPcIPYZXVGR12+VBoSW46bSKVhFZ8thV/I6Sm/Xqmsz46LJNCETODyOvtFYAnagYUBTq5sbBznovWJNaRkM38fQII+oXV3V1Ku9Y10kPXrQL0JwRoNvzrvAzZJBLGKArdBB9yeVgfLAp3NwmZAwawSSBfh73sBqcTgfrZvjN95xvJWfFvveZINV1Fb4EIfFCZJHcNWNLG8d0PEsk5TjFqKuTjkgYWP5xogiepN8BJfPB+QPfdTPlWr+Gos2pDgo83sna5NehHowgkDiA==";
        let pubkey_json = r#"{"id":"https://hotdog.place/users/renning#main-key","owner":"https://hotdog.place/users/renning","publicKeyPem":"-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAokhkD5QZh/eEb1mB9NRx\nfEm/aK05jSveg3X43s8LVoPQYY4030ql+IfHnsRtEJuzH5VWsYovjweT7ButDRX2\nAmk8IS94cqF7frDPDfBrNKJXfapmL7d3VuXU+BGOfLJZBK0NaEXvLK+Tssla4u+G\nUNinYnbOjnXvDOEkTOVpwTpcutHWSZrOcI8AdBXU3dv/c57sKXoIDZbVF9ZWEudL\n6/LsW0bpvXcBDPq1njOC9/WQcgtoe40WF6tROopyTZ/J+jlIKDuySW2/tsTrP6lg\nQ9TBzkj19leFDvCo6oWZ8aD6z8k5N6/ZAVjFtnivujc4rcoyPDPZArhIEP3n6R0d\n2QIDAQAB\n-----END PUBLIC KEY-----\n"}"#;
        let comparison = r#"(request-target): post /users/test2/inbox
host: ap.rens.page
date: Mon, 04 Sep 2023 20:49:38 GMT
digest: SHA-256=x0QZ2hdf3slWOdA4/DyxLEv4uEzU/FgjP9ho8EzR8sk=
content-type: application/activity+json"#;

        let pubkey: PublicKey = serde_json::from_str(pubkey_json).unwrap();
        let signature = base64_decode(signature).unwrap();

        pubkey.verify(comparison.as_bytes(), &signature).unwrap();
    }

    #[test]
    fn test_ruby_generated_signature() {
        let sig = "IwFW79lM9f7d0fZjHorNs2pPpAhMunJE9x2MYPIkVmS2XcCusRxD37sjjY6neI56x9MJt0iQLQpYcutHLYq1TYA1e4LYuARAleC0jrBh5uN1EKZrQ39htz0iKLbkl2U+zjl09c0rPN98KNGPZKgPSaJg5yEwqYYKAseluhiQRt5uuVJJZEk1E/b1KwZW0/U4QQFclu0vixq5hFi7vRwP8PWZV/VzCoCk/jS6/2P8O02ol+iZkdvKVgd2eO4phHsjeD6pZMbLnGoZ+aODLBFys7h7pcYRn1smCoDppbAb3xbQqpzLJ9ZLLv5B5r+BO9KFk6NLWd5xuV8IXcEjowIe5w==";
        let sig = base64_decode(sig).unwrap();

        let pubkey_json = r#"{"id":"","owner":"","publicKeyPem":"-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAsySwvshQIlMYLP4O/a/i\ncm25Jc7lOCx40WUYOVzIS/8YUeZw3mN5IfZURRWybB5ESwZCTKlqqgQs3s/WCCqD\ndER9BGLjph14ywCsSij4yFToHg4rAkzwnuiEBpjwb9TZxoWclQ6w7/L90zuphidA\nnCgSoxqNJ+L0xtJ92wf4vHQeuimKgna76I2VHFmgD9JOaD2ISL6+9D4v2lj6biNM\n/bXffipv6LxuM6p582BI2PH7OjBj617kd8DetYn71MpAMj3Kq8zhFQcbQwIpXIXe\nRYcQ8pCEQMHNYSLNhYfaFdgQJqy/OkUlIOrGIVA/XdVcznwHsmfzVgpZLQcG4gH6\nAwIDAQAB\n-----END PUBLIC KEY-----\n"}"#;
        let pubkey: PublicKey = serde_json::from_str(pubkey_json).unwrap();

        let comparison = "Hello world";
        pubkey.verify(comparison.as_bytes(), &sig).unwrap();
    }
}
