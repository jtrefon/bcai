use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct KeypairJson {
    pub public: String,
    pub secret: String,
}

/// Generate a new Ed25519 keypair encoded as hex strings.
pub fn generate_keypair() -> KeypairJson {
    let mut csprng = OsRng;
    let keypair = Keypair::generate(&mut csprng);
    KeypairJson {
        public: hex::encode(keypair.public.to_bytes()),
        secret: hex::encode(keypair.secret.to_bytes()),
    }
}
