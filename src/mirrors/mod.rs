use std::{sync::Arc, time::{Duration, Instant}};

use reqwest::header::HeaderMap;
use tokio::sync::RwLock;
use tracing::warn;

pub mod beatconnect;
pub mod catboy;
pub mod nerinyan;
pub mod osudirect;
pub mod sayobot;

#[derive(Default)]
pub struct RatelimitInfo {
    pub remaining: u32,
    pub reset_at: Option<Instant>,
}

#[derive(Default)]
pub struct Ratelimiter {
    pub info: Arc<RwLock<RatelimitInfo>>,
}

impl Ratelimiter {
    async fn wait_if_needed(&self) {
        if let Some(reset_at) = self.info.read().await.reset_at {
            if reset_at > Instant::now() {
                let wait_duration = reset_at.duration_since(Instant::now());
                warn!("You've hit an rate-limit, chill out, and wait for 60 seconds.");

                tokio::time::sleep(wait_duration).await;
            }
        }
    }

    async fn update_rate_limit(&self, headers: &HeaderMap) {
        if let Some(remaining) = headers.get("x-ratelimit-remaining").and_then(|v| v.to_str().ok()) {
            self.info.write().await.remaining = remaining.parse().unwrap_or(0);
        }
        
        if self.info.read().await.remaining <= 1 {
            self.info.write().await.reset_at = Some(Instant::now() + Duration::from_secs(70));
        }
    }
}

#[async_trait::async_trait]
pub trait Mirror {
    fn get_name(&self) -> &'static str;
    fn get_base_url(&self) -> &'static str;
    async fn get_file(&self, id: i32, rate_limiter: &Ratelimiter) -> Result<Vec<u8>, String>;
}