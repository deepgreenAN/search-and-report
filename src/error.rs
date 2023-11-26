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
    /// リクエストに関するエラー
    #[error("Error::RequestError: {0}")]
    RequestError(String),
    /// ファイルのI/Oに関するエラー
    #[error("Error::FileError: {0}")]
    FileError(String),
    /// OSに関するエラー
    #[error("Error::OsError: {0}")]
    OsError(String),
}

impl<'token> From<scraper::error::SelectorErrorKind<'token>> for Error {
    fn from(value: scraper::error::SelectorErrorKind<'token>) -> Self {
        Self::ScraperError(value.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::RequestError(value.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::FileError(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::FileError(value.to_string())
    }
}

impl From<notify_rust::error::Error> for Error {
    fn from(value: notify_rust::error::Error) -> Self {
        Self::OsError(value.to_string())
    }
}
