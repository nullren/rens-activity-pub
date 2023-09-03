use rsa::{pkcs8::EncodePublicKey, RsaPrivateKey};
use serde::{Deserialize, Serialize};

const BITS: usize = 2048;

#[derive(Serialize, Deserialize, Debug)]
pub struct Key {
    owner: String,
    #[serde(rename = "privateKey")]
    private_key: RsaPrivateKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicKey {
    id: String,
    owner: String,
    #[serde(rename = "publicKeyPem")]
    public_key_pem: String,
}

impl Key {
    pub fn new(owner: String) -> Self {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, BITS).unwrap();
        Self { owner, private_key }
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey {
            id: format!("{}/#main-key", self.owner),
            owner: self.owner.clone(),
            public_key_pem: self
                .private_key
                .to_public_key()
                .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
                .unwrap(),
        }
    }
}
