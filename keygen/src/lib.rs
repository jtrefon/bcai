use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

// Export the Keypair type for use in the binary
pub use ed25519_dalek::Keypair as Ed25519Keypair;

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

/// Generate a new Ed25519 keypair returning the raw keypair.
pub fn generate_ed25519_keypair() -> Keypair {
    let mut csprng = OsRng;
    Keypair::generate(&mut csprng)
}
