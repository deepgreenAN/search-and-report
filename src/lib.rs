pub mod error;
mod parser;
mod platform;
mod request;

use chrono::{NaiveDate, NaiveTime};

/// ポストを表す型
#[derive(Clone, PartialEq, Eq)]
pub struct Post {
    pub author: String,
    pub date: NaiveDate,
    pub time: Option<NaiveTime>,
    pub content: String,
}

pub type Posts = Vec<Post>;
