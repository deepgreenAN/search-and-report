use crate::error::Error;
use crate::PostParser;
use crate::{Post, Posts};

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime};
use chrono_tz::Asia::Tokyo;

use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};
use tracing::debug;

static DATETIME_PAT_1: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d{1,2}):(\d{1,2})").unwrap());
static DATETIME_PAT_2: Lazy<Regex> = Lazy::new(|| Regex::new(r"昨日(\d{1,2}):(\d{1,2})").unwrap());
static DATETIME_PAT_3: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d{1,2})月(\d{1,2})日\([月,火,水,木,金,土,日]\)(\d{1,2}):(\d{1,2})").unwrap()
});
static DATETIME_PAT_4: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\d{4})年(\d{1,2})月(\d{1,2})日").unwrap());

/// 東京の今日の日付を取得
fn today_jp() -> NaiveDate {
    chrono::Utc::now().with_timezone(&Tokyo).date_naive()
}

/// 時間のパーサー．東京時間からローカルに変換する必要がある．
fn yahoojp_time_parser(datetime_str: &str) -> Result<(NaiveDate, Option<NaiveTime>), Error> {
    let trim_pat: &[_] = &['\n', ' '];
    let trimmed = datetime_str.replace(trim_pat, "");

    debug!("trimmed datetime string: {}", trimmed);

    if let Some(captures) = DATETIME_PAT_1.captures(&trimmed) {
        debug!("PAT_1, captures: {:?}", captures);

        let time_jp = {
            let hour = captures[1]
                .parse::<u32>()
                .map_err(|e| Error::ParseDatetimeError(e.to_string()))?;
            let min = captures[2]
                .parse::<u32>()
                .map_err(|e| Error::ParseDatetimeError(e.to_string()))?;
            NaiveTime::from_hms_opt(hour, min, 0)
                .ok_or(Error::ParseDatetimeError("unexpected time".to_string()))?
        };

        let datetime_jp = NaiveDateTime::new(today_jp(), time_jp)
            .and_local_timezone(Tokyo)
            .unwrap(); // ?
        let datetime_local = datetime_jp.with_timezone(&Local);

        Ok((datetime_local.date_naive(), Some(datetime_local.time())))
    } else if let Some(captures) = DATETIME_PAT_2.captures(&trimmed) {
        debug!("PAT_2, captures: {:?}", captures);

        let time_jp = {
            let hour = captures[1]
                .parse::<u32>()
                .map_err(|e| Error::ParseDatetimeError(e.to_string()))?;
            let min = captures[2]
                .parse::<u32>()
                .map_err(|e| Error::ParseDatetimeError(e.to_string()))?;
            NaiveTime::from_hms_opt(hour, min, 0)
                .ok_or(Error::ParseDatetimeError("unexpected time".to_string()))?
        };

        // 日付は昨日である．
        let datetime_jp = NaiveDateTime::new(today_jp() - chrono::Duration::days(1), time_jp)
            .and_local_timezone(Tokyo)
            .unwrap();

        let datetime_local = datetime_jp.with_timezone(&Local);

        Ok((datetime_local.date_naive(), Some(datetime_local.time())))
    } else if let Some(captures) = DATETIME_PAT_3.captures(&trimmed) {
        debug!("PAT_3, captures: {:?}", captures);

        let date_jp = {
            let this_year = today_jp().year();
            let month = captures[1]
                .parse::<u32>()
                .map_err(|e| Error::ParseDatetimeError(e.to_string()))?;
            let day = captures[2]
                .parse::<u32>()
                .map_err(|e| Error::ParseDatetimeError(e.to_string()))?;
            NaiveDate::from_ymd_opt(this_year, month, day)
                .ok_or(Error::ParseDatetimeError("unexpected date".to_string()))?
        };
        let time_jp = {
            let hour = captures[3]
                .parse::<u32>()
                .map_err(|e| Error::ParseDatetimeError(e.to_string()))?;
            let min = captures[4]
                .parse::<u32>()
                .map_err(|e| Error::ParseDatetimeError(e.to_string()))?;
            NaiveTime::from_hms_opt(hour, min, 0)
                .ok_or(Error::ParseDatetimeError("unexpected time".to_string()))?
        };
        let datetime_jp = NaiveDateTime::new(date_jp, time_jp)
            .and_local_timezone(Tokyo)
            .unwrap();
        let datetime_local = datetime_jp.with_timezone(&Local);

        Ok((datetime_local.date_naive(), Some(datetime_local.time())))
    } else if let Some(captures) = DATETIME_PAT_4.captures(&trimmed) {
        debug!("PAT_4, captures: {:?}", captures);
        let date = {
            let year = captures[1]
                .parse::<i32>()
                .map_err(|e| Error::ParseDatetimeError(e.to_string()))?;
            let month = captures[2]
                .parse::<u32>()
                .map_err(|e| Error::ParseDatetimeError(e.to_string()))?;
            let day = captures[3]
                .parse::<u32>()
                .map_err(|e| Error::ParseDatetimeError(e.to_string()))?;
            NaiveDate::from_ymd_opt(year, month, day)
                .ok_or(Error::ParseDatetimeError("unexpected date".to_string()))?
        };
        // ローカルも一緒としかできない．
        Ok((date, None))
    } else {
        Err(Error::ParseDatetimeError(format!(
            "Unexpected string: {}",
            trimmed
        )))
    }
}

/// Yahoojpに対応したパーサー
pub struct YahooJpParser;

impl PostParser for YahooJpParser {
    fn parse(source: String) -> Result<Posts, Error> {
        let document = Html::parse_document(&source);

        let body_container_selector = Selector::parse(r#"div[class^=Tweet_bodyContainer]"#)?;
        let author_name_selector = Selector::parse(r#"span[class^=Tweet_authorName]"#)?;
        let content_selector = Selector::parse(r#"div[class^=Tweet_body]"#)?;
        let datetime_selector = Selector::parse(r#"time[class^=Tweet_time] > a"#)?;

        let mut posts: Vec<Post> = Vec::new();

        for body_container in document.select(&body_container_selector) {
            let author_name = body_container.select(&author_name_selector).next().ok_or(
                Error::UnexpectedStructureError {
                    selector: "div[class^=Tweet_bodyContainer] span[class^=Tweet_authorName]"
                        .to_string(),
                },
            )?;
            let datetime = body_container.select(&datetime_selector).next().ok_or(
                Error::UnexpectedStructureError {
                    selector: "div[class^=Tweet_bodyContainer] time[class^=Tweet_time] > a"
                        .to_string(),
                },
            )?;
            let content = body_container.select(&content_selector).next().ok_or(
                Error::UnexpectedStructureError {
                    selector: "div[class^=Tweet_bodyContainer] div[class^=Tweet_body]".to_string(),
                },
            )?;

            let mut content_buffer = String::new();

            for content_text in content.text() {
                content_buffer.push_str(content_text);
            }

            let (date, time) = yahoojp_time_parser(&datetime.inner_html())?;

            posts.push(Post {
                author: author_name.inner_html(),
                date,
                time,
                content: content_buffer,
            })
        }

        Ok(posts)
    }
}

#[cfg(test)]
mod test {
    use super::{today_jp, yahoojp_time_parser};
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn test_time_parser() {
        use chrono::{NaiveDate, NaiveTime};

        assert_eq!(
            yahoojp_time_parser("0:17").unwrap(),
            (today_jp(), Some(NaiveTime::from_hms_opt(0, 17, 0).unwrap()))
        );

        assert_eq!(
            yahoojp_time_parser(
                "昨日\n                                                                    17:12"
            )
            .unwrap(),
            (
                today_jp() - chrono::Duration::days(1),
                Some(NaiveTime::from_hms_opt(17, 12, 0).unwrap())
            )
        );

        assert_eq!(
            yahoojp_time_parser("11月24日(金)\n                                                                3:00").unwrap(),
            (NaiveDate::from_ymd_opt(2023, 11, 24).unwrap(), Some(NaiveTime::from_hms_opt(3, 0, 0).unwrap()))
        );

        assert_eq!(
            yahoojp_time_parser("2023年11月26日").unwrap(),
            (NaiveDate::from_ymd_opt(2023, 11, 26).unwrap(), None)
        );
    }
}
