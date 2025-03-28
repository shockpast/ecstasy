use serde::Deserialize;

use super::Mirror;

#[derive(Debug, Clone, Deserialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
pub struct Catboy;

#[async_trait::async_trait]
impl Mirror for Catboy {
    fn get_name(&self) -> &'static str {
        "catboy.best"
    }

    fn get_base_url(&self) -> &'static str {
        "https://catboy.best/d"
    }

    async fn get_file(&self, id: i32) -> Result<Vec<u8>, String> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/{}", self.get_base_url(), id))
            .header("User-Agent", "shockpast/osu-collector-cli: 1.0.0")
            .send()
            .await
            .unwrap();

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let bytes = response.bytes().await.map_err(|e| e.to_string())?;

        if content_type.contains("application/json") {
            if let Ok(json) = serde_json::from_slice::<ErrorResponse>(&bytes) {
                return Err(json.error);
            }
        }

        Ok(bytes.to_vec())
    }
}
