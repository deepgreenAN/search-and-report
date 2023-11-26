use crate::Report;
use crate::{error::Error, Posts};

use notify_rust::Notification;

/// Osの通知を用いたリポーター
pub struct NotificationReporter {
    content: String,
}

impl NotificationReporter {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}

#[async_trait::async_trait]
impl Report for NotificationReporter {
    async fn report(&self, _: &Posts) -> Result<(), Error> {
        Notification::new()
            .summary("Search and Reporter Notification")
            .body(&self.content)
            .show()?;

        Ok(())
    }
}
