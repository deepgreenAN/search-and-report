use chrono::Timelike;

use crate::Report;
use crate::{error::Error, Posts};

use std::borrow::Cow;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::info;

/// Postsの内容を全てjsonに保存するリポーター．
pub struct JsonSaveReporter {
    dir_path: PathBuf,
}

impl JsonSaveReporter {
    pub fn new<'a, P: Into<Cow<'a, Path>>>(dir_path: P) -> Self {
        let dir_path: Cow<'a, Path> = dir_path.into();

        Self {
            dir_path: dir_path.into_owned(),
        }
    }
}

#[async_trait::async_trait]
impl Report for JsonSaveReporter {
    async fn report(&self, posts: &Posts) -> Result<(), Error> {
        use chrono::Datelike;

        let json_string = serde_json::to_string_pretty(posts)?;

        // ディレクトリの存在確認，作製
        if !self.dir_path.is_dir() {
            std::fs::DirBuilder::new()
                .recursive(true)
                .create(&self.dir_path)?
        }

        let file_path = {
            let now = chrono::Local::now();
            let file_name = format!(
                "report_{}_{}_{}_{}_{}_{:.0}.json",
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                now.second()
            );

            let mut file_path = self.dir_path.clone();
            file_path.push(file_name);
            file_path
        };

        info!("Creating and saving into: {:?}", file_path);
        let mut file = File::create(&file_path)?;
        file.write_all(json_string.as_bytes())?;

        Ok(())
    }
}
