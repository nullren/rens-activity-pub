use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::pkcs8::{
    DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding,
};
use rsa::signature::{RandomizedSigner, SignatureEncoding, Verifier};
use rsa::{RsaPrivateKey, RsaPublicKey};
use sha2::Sha256;
use std::error::Error;

const KEY_SIZE: usize = 2048;
pub fn generate_keypair() -> Result<(String, String), Box<dyn Error>> {
    let mut rng = rand::thread_rng();
    let bits = KEY_SIZE;
    let private_key = RsaPrivateKey::new(&mut rng, bits)?;
    let private_key_pem = private_key.to_pkcs8_pem(LineEnding::LF)?;
    let public_key_pem = private_key
        .to_public_key()
        .to_public_key_pem(LineEnding::LF)?;
    Ok((private_key_pem.to_string(), public_key_pem))
}

pub fn sign<S, T>(key_pem: S, data: T) -> Result<Vec<u8>, Box<dyn Error>>
where
    S: AsRef<str>,
    T: AsRef<[u8]>,
{
    let key = RsaPrivateKey::from_pkcs8_pem(key_pem.as_ref())?;
    let signer = SigningKey::<Sha256>::new(key);
    let mut rng = rand::thread_rng();
    let sig = signer.try_sign_with_rng(&mut rng, data.as_ref())?;
    Ok(sig.to_vec())
}

pub fn verify<S, T>(key_pem: S, msg: T, sig: T) -> Result<(), Box<dyn Error>>
where
    S: AsRef<str>,
    T: AsRef<[u8]>,
{
    let key = RsaPublicKey::from_public_key_pem(key_pem.as_ref())?;
    let sig = Signature::try_from(sig.as_ref())?;
    let verifier = VerifyingKey::<Sha256>::new(key);
    verifier.verify(msg.as_ref(), &sig)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::utils::base64_decode;

    #[test]
    fn test_ruby_generated_signature() {
        let pubkey = "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAsySwvshQIlMYLP4O/a/i\ncm25Jc7lOCx40WUYOVzIS/8YUeZw3mN5IfZURRWybB5ESwZCTKlqqgQs3s/WCCqD\ndER9BGLjph14ywCsSij4yFToHg4rAkzwnuiEBpjwb9TZxoWclQ6w7/L90zuphidA\nnCgSoxqNJ+L0xtJ92wf4vHQeuimKgna76I2VHFmgD9JOaD2ISL6+9D4v2lj6biNM\n/bXffipv6LxuM6p582BI2PH7OjBj617kd8DetYn71MpAMj3Kq8zhFQcbQwIpXIXe\nRYcQ8pCEQMHNYSLNhYfaFdgQJqy/OkUlIOrGIVA/XdVcznwHsmfzVgpZLQcG4gH6\nAwIDAQAB\n-----END PUBLIC KEY-----\n";
        let comparison = "Hello world";
        let sig = "IwFW79lM9f7d0fZjHorNs2pPpAhMunJE9x2MYPIkVmS2XcCusRxD37sjjY6neI56x9MJt0iQLQpYcutHLYq1TYA1e4LYuARAleC0jrBh5uN1EKZrQ39htz0iKLbkl2U+zjl09c0rPN98KNGPZKgPSaJg5yEwqYYKAseluhiQRt5uuVJJZEk1E/b1KwZW0/U4QQFclu0vixq5hFi7vRwP8PWZV/VzCoCk/jS6/2P8O02ol+iZkdvKVgd2eO4phHsjeD6pZMbLnGoZ+aODLBFys7h7pcYRn1smCoDppbAb3xbQqpzLJ9ZLLv5B5r+BO9KFk6NLWd5xuV8IXcEjowIe5w==";
        let sig = base64_decode(sig).unwrap();

        super::verify(pubkey, comparison.as_bytes(), &sig).unwrap();
    }
}
