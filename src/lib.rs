pub mod error;
pub mod parser;
pub mod predicates;
pub mod reporter;
pub mod request;

pub mod platforms;

pub use parser::PostParser;
pub use reporter::Report;
pub use request::RequestSource;

use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

/// ポストを表す型
#[derive(Clone, PartialEq, Eq, Serialize, Default)]
pub struct Post {
    /// アカウント名
    pub author: String,
    /// ローカルの日時
    pub date: NaiveDate,
    /// ローカルの時間(Option)
    pub time: Option<NaiveTime>,
    /// ポストの内容
    pub content: String,
}

pub type Posts = Vec<Post>;

/// プラットフォームやバージョン管理用のためのトレイト
pub trait PlatForm {
    type Parser: PostParser;
    type Requester: RequestSource;
}

/// 検索・リポート設定
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchConfig {
    pub keywords: Vec<String>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig {
            keywords: vec!["Rust".to_string()],
        }
    }
}

/// 検索とリポートを行う公開API
pub async fn search_and_report<T: PlatForm, R: Report, P: Fn(&Posts) -> bool>(
    config: &SearchConfig,
    _platform: T,
    reporter: &R,
    pred: P,
) -> Result<(), error::Error> {
    let source = T::Requester::request(&config.keywords).await?;

    let posts = T::Parser::parse(source)?;

    if pred(&posts) {
        reporter.report(&posts).await?;
    }

    Ok(())
}
