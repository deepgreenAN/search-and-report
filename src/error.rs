/// アプリケーション・ライブラリとして利用したときのエラー
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// scraperに関するエラー
    #[error("Error::ScraperError:{0}")]
    ScraperError(String),
    /// リクエストして取得したソースが意図しないものであった場合のエラー
    #[error("Error::UnexpectedStructureError: {selector} is not found in source.")]
    UnexpectedStructureError { selector: String },
    /// Datetimeのパースに関するエラー
    #[error("Error::ParseDatetimeError: {0}")]
    ParseDatetimeError(String),
}

impl<'token> From<scraper::error::SelectorErrorKind<'token>> for Error {
    fn from(value: scraper::error::SelectorErrorKind<'token>) -> Self {
        Self::ScraperError(value.to_string())
    }
}
