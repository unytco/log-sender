//! Log-collector low-level http client.

use super::*;

pub use reqwest;

/// Log-collector low-level http client.
pub struct Client {
    client: reqwest::Client,
    url: reqwest::Url,
}

impl Client {
    /// Construct a new [Client] instance.
    pub async fn new(url: reqwest::Url) -> Result<Self> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(std::io::Error::other)?;

        Ok(Self { client, url })
    }

    /// Make a "health" call.
    pub async fn health(&self) -> Result<()> {
        let mut url = self.url.clone();
        url.set_path("/");

        #[derive(serde::Deserialize)]
        struct R {
            status: String,
        }

        let res: R = self
            .client
            .get(url)
            .send()
            .await
            .map_err(std::io::Error::other)?
            .json()
            .await
            .map_err(std::io::Error::other)?;

        if res.status != "healthy" {
            return Err(std::io::Error::other(format!(
                "bad server status: {}",
                res.status
            )));
        }

        Ok(())
    }

    /// Make a "drone-registration" call.
    pub async fn drone_registration(
        &self,
        pk: &PubKey,
        sk: &SecKey,
    ) -> Result<u64> {
        let mut url = self.url.clone();
        url.set_path("/drone-registration");

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Req {
            drone_pub_key: String,
            unyt_pub_key: String,
            drone_signature: String,
            signature_timestamp: u64,
        }

        let drone_pub_key = pk.encode()?;
        let unyt_pub_key = drone_pub_key.clone();
        let signature_timestamp = std::time::SystemTime::UNIX_EPOCH
            .elapsed()
            .expect("can get time")
            .as_millis() as u64;

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Sig {
            drone_pub_key: String,
            unyt_pub_key: String,
            timestamp: u64,
        }

        let sig = serde_json::to_string(&Sig {
            drone_pub_key: drone_pub_key.clone(),
            unyt_pub_key: unyt_pub_key.clone(),
            timestamp: signature_timestamp,
        })?;

        let drone_signature = sk.sign(sig.as_bytes())?;

        #[derive(Debug, serde::Deserialize)]
        struct Reg {
            id: u64,
        }

        #[derive(Debug, serde::Deserialize)]
        struct Res {
            success: bool,
            registration: Reg,
        }

        let res: Res = self
            .client
            .post(url)
            .json(&Req {
                drone_pub_key,
                unyt_pub_key,
                drone_signature,
                signature_timestamp,
            })
            .send()
            .await
            .map_err(std::io::Error::other)?
            .json()
            .await
            .map_err(std::io::Error::other)?;

        if res.success {
            return Ok(res.registration.id);
        }

        Err(std::io::Error::other(format!("invalid response: {res:?}")))
    }
}
