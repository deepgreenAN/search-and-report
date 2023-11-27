use crate::error::Error;
use crate::PostParser;
use crate::{Post, Posts};

use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime};
use chrono_tz::Asia::Tokyo;

use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};
use tracing::{debug, info};

static DATETIME_PAT_1: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d{1,2})秒前").unwrap());
static DATETIME_PAT_2: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d{1,2})分前").unwrap());
static DATETIME_PAT_3: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d{1,2}):(\d{1,2})").unwrap());
static DATETIME_PAT_4: Lazy<Regex> = Lazy::new(|| Regex::new(r"昨日(\d{1,2}):(\d{1,2})").unwrap());
static DATETIME_PAT_5: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d{1,2})月(\d{1,2})日\([月,火,水,木,金,土,日]\)(\d{1,2}):(\d{1,2})").unwrap()
});
static DATETIME_PAT_6: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\d{4})年(\d{1,2})月(\d{1,2})日").unwrap());

/// 東京の現在時刻を取得
fn now_jp() -> NaiveDateTime {
    let datetime_jp = chrono::Utc::now().with_timezone(&Tokyo);
    NaiveDateTime::new(datetime_jp.date_naive(), datetime_jp.time())
}

/// 時間のパーサー．東京時間からローカルに変換する必要がある．
fn yahoojp_time_parser(
    datetime_str: &str,
    now_jp: NaiveDateTime,
) -> Result<(NaiveDate, Option<NaiveTime>), Error> {
    let trim_pat: &[_] = &['\n', ' '];
    let trimmed = datetime_str.replace(trim_pat, "");

    debug!("trimmed datetime string: {}", trimmed);

    if let Some(captures) = DATETIME_PAT_1.captures(&trimmed) {
        debug!("PAT_1, captures: {:?}", captures);

        let duration_second = {
            let second = captures[1]
                .parse::<i64>()
                .map_err(|e| Error::ParseDatetimeError(e.to_string()))?;
            Duration::seconds(second)
        };

        let datetime_jp = (now_jp - duration_second)
            .and_local_timezone(Tokyo)
            .unwrap();
        let datetime_local = datetime_jp.with_timezone(&Local);

        Ok((datetime_local.date_naive(), Some(datetime_local.time())))
    } else if let Some(captures) = DATETIME_PAT_2.captures(&trimmed) {
        debug!("PAT_2, captures: {:?}", captures);

        let duration_minutes = {
            let minutes = captures[1]
                .parse::<i64>()
                .map_err(|e| Error::ParseDatetimeError(e.to_string()))?;
            Duration::minutes(minutes)
        };

        let datetime_jp = (now_jp - duration_minutes)
            .and_local_timezone(Tokyo)
            .unwrap();
        let datetime_local = datetime_jp.with_timezone(&Local);

        Ok((datetime_local.date_naive(), Some(datetime_local.time())))
    } else if let Some(captures) = DATETIME_PAT_3.captures(&trimmed) {
        debug!("PAT_3, captures: {:?}", captures);

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

        let datetime_jp = NaiveDateTime::new(now_jp.date(), time_jp)
            .and_local_timezone(Tokyo)
            .unwrap();
        let datetime_local = datetime_jp.with_timezone(&Local);

        Ok((datetime_local.date_naive(), Some(datetime_local.time())))
    } else if let Some(captures) = DATETIME_PAT_4.captures(&trimmed) {
        debug!("PAT_4, captures: {:?}", captures);

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

        let datetime_jp = NaiveDateTime::new(now_jp.date() - Duration::days(1), time_jp)
            .and_local_timezone(Tokyo)
            .unwrap();

        let datetime_local = datetime_jp.with_timezone(&Local);

        Ok((datetime_local.date_naive(), Some(datetime_local.time())))
    } else if let Some(captures) = DATETIME_PAT_5.captures(&trimmed) {
        debug!("PAT_5, captures: {:?}", captures);

        let date_jp = {
            let this_year = now_jp.year();
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
    } else if let Some(captures) = DATETIME_PAT_6.captures(&trimmed) {
        debug!("PAT_6, captures: {:?}", captures);
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
        info!("Parsing html source");
        let document = Html::parse_document(&source);
        let now_jp = now_jp();

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

            let (date, time) = yahoojp_time_parser(&datetime.inner_html(), now_jp)?;

            posts.push(Post {
                author: author_name.inner_html(),
                date,
                time,
                content: content_buffer,
            })
        }
        info!("Finished parsing source html.");

        Ok(posts)
    }
}

#[cfg(test)]
mod test {
    use super::{now_jp, yahoojp_time_parser};
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn test_time_parser() {
        use chrono::{Duration, Local, NaiveDate, NaiveDateTime, NaiveTime};

        let now_jp = now_jp();
        let now_local = {
            let local = now_jp.and_local_timezone(Local).unwrap();
            NaiveDateTime::new(local.date_naive(), local.time())
        };

        {
            let datetime_local = now_local - Duration::seconds(25);
            assert_eq!(
                yahoojp_time_parser("25秒前", now_jp).unwrap(),
                (datetime_local.date(), Some(datetime_local.time()))
            );
        }

        {
            let datetime_local = now_local - Duration::minutes(5);
            assert_eq!(
                yahoojp_time_parser("5分前", now_jp).unwrap(),
                (datetime_local.date(), Some(datetime_local.time()))
            );
        }

        assert_eq!(
            yahoojp_time_parser("0:17", now_jp).unwrap(),
            (
                now_local.date(),
                Some(NaiveTime::from_hms_opt(0, 17, 0).unwrap())
            )
        );

        assert_eq!(
            yahoojp_time_parser(
                "昨日\n                                                                    17:12",
                now_jp
            )
            .unwrap(),
            (
                now_local.date() - chrono::Duration::days(1),
                Some(NaiveTime::from_hms_opt(17, 12, 0).unwrap())
            )
        );

        assert_eq!(
            yahoojp_time_parser("11月24日(金)\n                                                                3:00",
                now_jp
            ).unwrap(),
            (NaiveDate::from_ymd_opt(2023, 11, 24).unwrap(), Some(NaiveTime::from_hms_opt(3, 0, 0).unwrap()))
        );

        assert_eq!(
            yahoojp_time_parser("2023年11月26日", now_jp).unwrap(),
            (NaiveDate::from_ymd_opt(2023, 11, 26).unwrap(), None)
        );
    }
}
