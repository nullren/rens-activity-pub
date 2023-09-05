use rsa::pkcs8::{DecodePublicKey, EncodePublicKey};
use rsa::pss::{Signature, SigningKey, VerifyingKey};
use rsa::signature::{RandomizedSigner, SignatureEncoding, Verifier};
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::error::Error;

const BITS: usize = 2048;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Key {
    owner: String,
    #[serde(rename = "privateKey")]
    private_key: RsaPrivateKey,
}

impl Key {
    pub fn new(owner: String) -> Result<Self, Box<dyn Error>> {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, BITS)?;
        Ok(Self { owner, private_key })
    }

    pub fn public_key(&self) -> Result<PublicKey, Box<dyn Error>> {
        Ok(PublicKey {
            id: format!("{}/#main-key", self.owner),
            owner: self.owner.clone(),
            public_key_pem: self
                .private_key
                .to_public_key()
                .to_public_key_pem(rsa::pkcs8::LineEnding::LF)?,
        })
    }

    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut rng = rand::thread_rng();
        let signer = SigningKey::<Sha256>::new(self.private_key.clone());
        let sig = signer.try_sign_with_rng(&mut rng, data)?;
        Ok(sig.to_vec())
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

    pub fn to_rsa_public_key(&self) -> Result<rsa::RsaPublicKey, Box<dyn Error>> {
        Ok(RsaPublicKey::from_public_key_pem(&self.public_key_pem)?)
    }

    pub fn verify(&self, data: &[u8], sig: &[u8]) -> Result<(), Box<dyn Error>> {
        let public_key = self.to_rsa_public_key()?;
        let sig: Signature = sig.try_into()?;
        let verifier = VerifyingKey::<Sha256>::new(public_key);
        verifier.verify(data, &sig)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
