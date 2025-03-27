pub mod beatconnect;
pub mod catboy;
pub mod nerinyan;
pub mod osudirect;

#[async_trait::async_trait]
pub trait Mirror {
    fn get_name(&self) -> &'static str;
    fn get_base_url(&self) -> &'static str;
    async fn get_file(&self, id: i32) -> Result<Vec<u8>, String>;
}
