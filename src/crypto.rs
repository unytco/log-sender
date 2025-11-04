//! Crypto utility types.

use super::*;
use base64::prelude::*;

/// Public key.
pub struct PubKey(rsa::RsaPublicKey);

impl PubKey {
    /// Encode the public key in SPKI format for compatibility with Web Crypto API.
    pub fn encode(&self) -> Result<String> {
        use rsa::pkcs8::EncodePublicKey;
        // Use SPKI format which is required by Web Crypto API for RSA-PSS verification
        Ok(BASE64_STANDARD.encode(
            self.0
                .to_public_key_pem(rsa::pkcs8::LineEnding::default())
                .map_err(std::io::Error::other)?
                .as_bytes(),
        ))
    }
}

/// Secret key.
pub struct SecKey(rsa::RsaPrivateKey);

impl SecKey {
    /// Decode the private key.
    pub fn decode(data: &[u8]) -> Result<Self> {
        use rsa::pkcs8::DecodePrivateKey;
        Ok(Self(
            rsa::RsaPrivateKey::from_pkcs8_der(
                &BASE64_STANDARD
                    .decode(data)
                    .map_err(std::io::Error::other)?,
            )
            .map_err(std::io::Error::other)?,
        ))
    }

    /// Encode the private key.
    pub fn encode(&self) -> Result<String> {
        use rsa::pkcs8::EncodePrivateKey;
        Ok(BASE64_STANDARD.encode(
            self.0
                .to_pkcs8_der()
                .map_err(std::io::Error::other)?
                .as_bytes(),
        ))
    }

    /// Precompute to speed up runtime.
    pub async fn precompute(mut self) -> Result<Self> {
        tokio::task::spawn_blocking(|| {
            self.0.precompute().map_err(std::io::Error::other)?;
            Ok(self)
        })
        .await?
    }

    /// Sign some data.
    pub fn sign(&self, data: &[u8]) -> Result<String> {
        use rsa::sha2::Digest;
        let digest = rsa::sha2::Sha256::digest(data);
        let pss = rsa::pss::Pss::new_with_salt::<rsa::sha2::Sha256>(32);
        Ok(BASE64_STANDARD.encode(
            &self
                .0
                .sign_with_rng(&mut rand::thread_rng(), pss, &digest)
                .map_err(std::io::Error::other)?,
        ))
    }
}

/// Generate a keypair.
pub async fn generate_keypair() -> Result<(PubKey, SecKey)> {
    tokio::task::spawn_blocking(|| {
        let sk = rsa::RsaPrivateKey::new(&mut rand::thread_rng(), 2048)
            .map_err(std::io::Error::other)?;
        let pk = rsa::RsaPublicKey::from(&sk);
        Ok((PubKey(pk), SecKey(sk)))
    })
    .await?
}
