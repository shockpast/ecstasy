use reqwest::Error;
use serde::Deserialize;

use super::Mirror;

#[derive(Deserialize)]
pub struct Beatconnect;

#[async_trait::async_trait]
impl Mirror for Beatconnect {
    fn get_name(&self) -> &'static str {
        "beatconnect.io"
    }

    fn get_base_url(&self) -> &'static str {
        "https://beatconnect.io/b"
    }

    async fn get_file(&self, id: i32) -> Result<Vec<u8>, Error> {
        let client = reqwest::Client::new();
        let result = client
            .get(format!("{}/{}", self.get_base_url(), id))
            .header("User-Agent", "shockpast/osu-collector-cli: 1.0.0")
            .send()
            .await?
            .bytes()
            .await?;

        Ok(result.to_vec())
    }
}
