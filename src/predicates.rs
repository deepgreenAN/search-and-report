use crate::Posts;

use chrono::{Duration, Local, NaiveDateTime};
use std::collections::VecDeque;

/// 時間当たりのポストの個数で判定
pub struct NumberPerDuration {
    n: usize,
    duration: Duration,
}

impl NumberPerDuration {
    pub fn new(n: usize, duration: Duration) -> Self {
        Self { n, duration }
    }
    /// 判定用のメソッド．ソートを伴うためO(N logN)
    pub fn predicate(&self, posts: &Posts) -> bool {
        let mut datetimes = posts
            .iter()
            .filter_map(|post| (&post.time).map(|time| NaiveDateTime::new(post.date, time)))
            .collect::<Vec<_>>();

        datetimes.sort();

        let mut queue = VecDeque::<&NaiveDateTime>::with_capacity(self.n);

        for datetime in datetimes.iter() {
            queue.push_back(&datetime);

            if queue.len() > self.n {
                queue.pop_front();
            }

            if queue.len() == self.n
                && **queue.get(self.n - 1).unwrap() - **queue.get(0).unwrap() < self.duration
            {
                return true;
            }
        }

        false
    }
}

/// 最後の(最新の)投稿時間で判定
pub struct LatestPostTime {
    duration: Duration,
}

impl LatestPostTime {
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }

    /// 判定用のメソッド．O(N)
    pub fn predicate(&self, posts: &Posts) -> bool {
        let now = Local::now().naive_local();

        if let Some(latest_post_time) = posts
            .iter()
            .filter_map(|post| (&post.time).map(|time| NaiveDateTime::new(post.date, time)))
            .max()
        {
            (now - latest_post_time) < self.duration
        } else {
            false // postsが存在しない
        }
    }
}
