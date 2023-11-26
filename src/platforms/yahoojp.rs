mod yahoojp_parser;
mod yahoojp_request;

pub use yahoojp_parser::YahooJpParser;
pub use yahoojp_request::YahooJpRequest;

use crate::PlatForm;

/// YahooJpの検索をおこなう．
pub struct YahooJp;

impl PlatForm for YahooJp {
    type Parser = YahooJpParser;
    type Requester = YahooJpRequest;
}
