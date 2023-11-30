use crate::Report;
use crate::{error::Error, Posts};

use chrono::NaiveDateTime;
use notify_rust::Notification;

/// Osの通知を用いたリポーター
pub struct StaticNotificationReporter {
    content: String,
}

impl StaticNotificationReporter {
    pub fn new<S: Into<String>>(content: S) -> Self {
        Self {
            content: content.into(),
        }
    }
}

#[async_trait::async_trait]
impl Report for StaticNotificationReporter {
    async fn report(&self, _: &Posts) -> Result<(), Error> {
        Notification::new()
            .summary("Search and Reporter Notification")
            .body(&self.content)
            .show()?;

        Ok(())
    }
}

/// 通知で最新ポストの内容を表示するリポーター
pub struct LatestPostNotificationReporter;

#[async_trait::async_trait]
impl Report for LatestPostNotificationReporter {
    async fn report(&self, posts: &Posts) -> Result<(), Error> {
        if let Some((latest_post_index, latest_datetime)) = posts
            .iter()
            .enumerate()
            .filter_map(|(i, post)| {
                post.time
                    .map(|time| (i, NaiveDateTime::new(post.date, time)))
            })
            .max_by(|(_, x), (_, y)| x.cmp(y))
        {
            let latest_post = posts.get(latest_post_index).unwrap(); // 存在は確定されているため
            let content = format!(
                r#"
{}: {}
{}
            "#,
                &latest_post.author, latest_datetime, &latest_post.content
            );

            Notification::new()
                .summary("Latest Post Notification")
                .body(&content)
                .show()?;

            Ok(())
        } else {
            Err(Error::NothingPostError)
        }
    }
}
