use reqwest::Error;
use serde::Deserialize;

use super::Mirror;

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

    async fn get_file(&self, id: i32) -> Result<Vec<u8>, Error> {
        let client = reqwest::Client::new();
        let result = client
            .get(format!("{}/{}?nh=1&nsb=1&nv=1", self.get_base_url(), id))
            .header("User-Agent", "shockpast/osu-collector-cli: 1.0.0")
            .send()
            .await?
            .bytes()
            .await?;

        Ok(result.to_vec())
    }
}
