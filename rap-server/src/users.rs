use axum::extract::Path;
use axum::Json;
use serde_json::{json, Value};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Actor {
    id: String,
}

pub async fn json(actor: Path<Actor>) -> Json<Value> {
    Json(json!({
        "@context": [
            "https://www.w3.org/ns/activitystreams",
            "https://w3id.org/security/v1"
        ],

        "id": "https://ap.rens.page/users/Ren",
        "type": "Person",
        "preferredUsername": "Ren",
        "inbox": "https://ap.rens.page/users/Ren/inbox",

        "publicKey": {
            "id": "https://ap.rens.page/users/Ren#main-key",
            "owner": "https://ap.rens.page/users/Ren",
            "publicKeyPem": "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA3HVjbmnBo/pEgmEe+Hre\nxR9GaVWUlIj49L5PvlgOZp2Q4qZP1+DLwkMwE7jIqeH1z3d/5AahWQdb/MmO+Khp\nOkEisUzcJJlaWTEsKNJ6IxLmYsxhdRAz2/RgwTAGMfvXiIowl++ZUXBwsMvztzDl\nZ9NWOvfZQqTXOGGafePLHIS+Nv/Eu1+/u2W8mZJA7uAahFjXMNbefDIkACWVbzIv\n/2VnYG28NACrshyjdtLQ6+8/ypgnbejehRcP/UIAGfBBNyhlhaHAmHCWNNfwoNpF\n3Vz3J+q/QV6flA3egdVHerz25+gSOraUrB2xKbDJqeGJZaZmq8Zn1EyxrQTJFutv\newIDAQAB\n-----END PUBLIC KEY-----\n"
        }
    }))
}
