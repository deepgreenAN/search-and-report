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
use serde::Serialize;

/// ポストを表す型
#[derive(Clone, PartialEq, Eq, Serialize)]
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
pub struct SearchConfig {
    keywords: Vec<String>,
}

/// 検索とリポートを行う公開API
pub async fn search_and_report<T: PlatForm, R: Report, P: Fn(&Posts) -> bool>(
    config: &SearchConfig,
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
