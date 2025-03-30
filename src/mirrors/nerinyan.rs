use serde::Deserialize;

use super::{Mirror, Ratelimiter};

#[derive(Deserialize)]
pub struct Nerinyan;

#[async_trait::async_trait]
impl Mirror for Nerinyan {
    fn get_name(&self) -> &'static str {
        "nerinyan.moe"
    }

    fn get_base_url(&self) -> &'static str {
        "https://api.nerinyan.moe/d"
    }

    async fn get_file(&self, id: i32, rate_limit: &Ratelimiter) -> Result<Vec<u8>, String> {
        rate_limit.wait_if_needed().await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/{}", self.get_base_url(), id))
            .header("User-Agent", "shockpast/ecstasy: 1.1.2")
            .send()
            .await
            .unwrap();

        rate_limit.update_rate_limit(response.headers()).await;

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let bytes = response.bytes().await.map_err(|e| e.to_string())?;

        if content_type.contains("application/json") {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                return Err(json.to_string());
            }
        }

        Ok(bytes.to_vec())
    }
}
