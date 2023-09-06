use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::pkcs8::DecodePublicKey;
use rsa::signature::{RandomizedSigner, SignatureEncoding, Verifier};
use rsa::{RsaPrivateKey, RsaPublicKey};
use sha2::Sha256;
use std::error::Error;

pub fn sign<T: AsRef<[u8]>>(
    private_key: RsaPrivateKey,
    data: T,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let signer = SigningKey::<Sha256>::new(private_key.clone());
    let mut rng = rand::thread_rng();
    let sig = signer.try_sign_with_rng(&mut rng, data)?;
    Ok(sig.to_vec())
}

pub fn verify<T: AsRef<[u8]>>(key_pem: &str, msg: T, sig: T) -> Result<(), Box<dyn Error>> {
    let key = RsaPublicKey::from_public_key_pem(key_pem)?;
    let sig = Signature::try_from(sig)?;
    let verifier = VerifyingKey::<Sha256>::new(key);
    verifier.verify(msg, &sig)?;
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
