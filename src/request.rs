mod yahoojp_request;

use crate::error::Error;

/// htmlなどのソースをリクエストするためのトレイト
#[async_trait::async_trait]
pub trait RequestSource {
    async fn request() -> Result<String, Error>;
}
