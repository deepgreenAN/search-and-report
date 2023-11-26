use crate::error::Error;
use crate::Posts;

/// 各プラットフォームごとにPostをパースするためのトレイト．
pub trait PostParser {
    /// パースしてPostsを取得する．
    fn parse(source: String) -> Result<Posts, Error>;
}
