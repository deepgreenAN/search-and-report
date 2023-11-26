use crate::Report;
use crate::{error::Error, Posts};

use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Postsの内容を全てjsonに保存するリポーター．
pub struct JsonSaveReporter {
    path: Path,
}

#[async_trait::async_trait]
impl Report for JsonSaveReporter {
    async fn report(&self, posts: &Posts) -> Result<(), Error> {
        let json_string = serde_json::to_string_pretty(posts)?;

        let mut file = File::create(&self.path)?;
        file.write_all(json_string.as_bytes())?;

        Ok(())
    }
}
