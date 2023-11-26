use crate::parser::PostParser;
use crate::request::RequestSource;

/// プラットフォームやバージョン管理用のトレイト
pub trait PlatForm {
    type Parser: PostParser;
    type Requester: RequestSource;
}
