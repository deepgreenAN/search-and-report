mod yahoojp_parser;
mod yahoojp_request;

use crate::PlatForm;

/// YahooJpの検索をおこなう．
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct YahooJp;

impl PlatForm for YahooJp {
    type Parser = yahoojp_parser::YahooJpParser;
    type Requester = yahoojp_request::YahooJpRequest;
}
