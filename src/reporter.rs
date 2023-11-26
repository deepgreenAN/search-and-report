mod json_save_reporter;
mod notification_reporter;

pub use json_save_reporter::JsonSaveReporter;
pub use notification_reporter::NotificationReporter;

use crate::error::Error;
use crate::Posts;

use std::sync::Arc;

/// リポートを行うトレイト
#[async_trait::async_trait]
pub trait Report {
    async fn report(&self, posts: &Posts) -> Result<(), Error>;
}

/// リポーターのリスト
pub struct ReporterList {
    inner_list: Vec<Arc<dyn Report + Send + Sync>>,
}

impl ReporterList {
    pub fn new() -> Self {
        Self {
            inner_list: Vec::new(),
        }
    }
    pub fn append_reporter<R: Report + Send + Sync + 'static>(&mut self, reporter: R) {
        self.inner_list
            .push(Arc::new(reporter) as Arc<dyn Report + Send + Sync>);
    }
}

#[async_trait::async_trait]
impl Report for ReporterList {
    async fn report(&self, posts: &Posts) -> Result<(), Error> {
        for reporter in self.inner_list.iter() {
            reporter.report(posts).await?;
        }
        Ok(())
    }
}
