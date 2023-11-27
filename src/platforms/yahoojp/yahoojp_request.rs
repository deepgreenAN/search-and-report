use crate::error::Error;
use crate::RequestSource;

use reqwest::Client;
use tracing::info;

/// Yahoojpに対応したリクエスト
pub struct YahooJpRequest;

#[async_trait::async_trait]
impl RequestSource for YahooJpRequest {
    async fn request(keywords: &[String]) -> Result<String, Error> {
        let concat_keyword = keywords.join(" ");

        let url = format!(
            r"https://search.yahoo.co.jp/realtime/search?p={}&ei=UTF-8&ifr=tl_sc",
            &concat_keyword
        );

        info!("Attempting request to {}.", url);
        let res = Client::new().get(&url).send().await?;
        let text = res.text().await?;

        info!("Finished request to {}.", url);
        Ok(text)
    }
}
