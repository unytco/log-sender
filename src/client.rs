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

        let res = self
            .client
            .get(url)
            .send()
            .await
            .map_err(std::io::Error::other)?;

        if res.error_for_status_ref().is_err() {
            return Err(std::io::Error::other(
                res.text().await.map_err(std::io::Error::other)?
            ));
        }

        let res: R = res.json()
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
        config: &RuntimeConfigFile,
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

        let drone_pub_key = config.drone_pub_key.clone();
        let unyt_pub_key = config.unyt_pub_key.clone();
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

        let drone_signature = config.rt_drone_sec_key.sign(sig.as_bytes())?;

        #[derive(Debug, serde::Deserialize)]
        struct Reg {
            id: u64,
        }

        #[derive(Debug, serde::Deserialize)]
        struct Res {
            success: bool,
            registration: Reg,
        }

        let res = self
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
            .map_err(std::io::Error::other)?;

        if res.error_for_status_ref().is_err() {
            return Err(std::io::Error::other(
                res.text().await.map_err(std::io::Error::other)?
            ));
        }

        let res: Res = res
            .json()
            .await
            .map_err(std::io::Error::other)?;

        if res.success {
            return Ok(res.registration.id);
        }

        Err(std::io::Error::other(format!("invalid response: {res:?}")))
    }

    /// Submit metrics to the endpoint.
    pub async fn metrics(
        &self,
        config: &RuntimeConfigFile,
        proofs: Vec<String>,
    ) -> Result<()> {
        let mut url = self.url.clone();
        url.set_path("/metrics");

        #[derive(Clone, serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct ReqMetric {
            value: u64,
            timestamp: u64,
            #[serde(rename = "registered_unit_index")]
            registered_unit_index: u64,
            proof: String,
        }

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Req {
            signing_pub_key: String,
            drone_pub_key: String,
            unyt_pub_key: String,
            metrics: Vec<ReqMetric>,
            signature: String,
            timestamp: u64,
        }

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Sig {
            drone_pub_key: String,
            metrics: Vec<ReqMetric>,
            signing_pub_key: String,
            timestamp: u64,
            unyt_pub_key: String,
        }

        let timestamp = std::time::SystemTime::UNIX_EPOCH
            .elapsed()
            .expect("can get time")
            .as_millis() as u64;

        let metrics: Vec<ReqMetric> = proofs.into_iter().map(|proof| {
            ReqMetric {
                value: 0,
                timestamp,
                registered_unit_index: 0,
                proof,
            }
        }).collect();

        let sig = serde_json::to_string(&Sig {
            drone_pub_key: config.drone_pub_key.clone(),
            metrics: metrics.clone(),
            signing_pub_key: config.drone_pub_key.clone(),
            timestamp,
            unyt_pub_key: config.unyt_pub_key.clone(),
        })?;

        let signature = config.rt_drone_sec_key.sign(sig.as_bytes())?;

        let res = self
            .client
            .post(url)
            .json(&Req {
                signing_pub_key: config.drone_pub_key.clone(),
                drone_pub_key: config.drone_pub_key.clone(),
                unyt_pub_key: config.unyt_pub_key.clone(),
                metrics,
                signature,
                timestamp,
            })
            .send()
            .await
            .map_err(std::io::Error::other)?;

        if res.error_for_status_ref().is_err() {
            return Err(std::io::Error::other(
                res.text().await.map_err(std::io::Error::other)?
            ));
        }

        #[derive(Debug, serde::Deserialize)]
        struct Res {
            success: bool,
        }

        let res: Res = res
            .json()
            .await
            .map_err(std::io::Error::other)?;

        if res.success {
            return Ok(());
        }

        Err(std::io::Error::other(format!("invalid response: {res:?}")))
    }
}
