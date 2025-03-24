use async_trait::async_trait;

pub mod catboy;
pub mod osudirect;
pub mod nerinyan;
pub mod beatconnect;

#[async_trait]
pub trait Mirror {
  fn get_name(&self) -> &'static str;
  fn get_base_url(&self) -> &'static str;
  async fn get_file(&self, id: i32) -> Result<Vec<u8>, reqwest::Error>;
}
