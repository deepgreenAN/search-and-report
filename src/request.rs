use crate::error::Error;

/// htmlなどのソースをリクエストするためのトレイト
#[async_trait::async_trait]
pub trait RequestSource {
    async fn request(keywords: &[String]) -> Result<String, Error>;
}
