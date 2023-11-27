/// 設定ファイルについて
mod config {
    use search_and_report::SearchConfig;

    use serde::{Deserialize, Serialize};

    /// Config読み込みのエラー
    #[derive(Debug, thiserror::Error)]
    #[error("ConfigError: {0}")]
    pub struct ConfigError(pub String);

    /// Configファイルの一要素．条件を複数指定した場合はORになる．
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub struct SearchAndReportConfig {
        #[serde(flatten)]
        pub search_config: SearchConfig,
        pub cron: String,
        pub condition_n_per_h: Option<u32>,
        pub condition_contain: Option<Vec<String>>,
        pub condition_latest_in_h: Option<u32>,
        pub report_json_dir: Option<String>,
        pub report_os_content: Option<String>,
    }

    /// このデフォルトはデフォルトのconfigファイルを作製する際に使われる．
    impl Default for SearchAndReportConfig {
        fn default() -> Self {
            SearchAndReportConfig {
                search_config: Default::default(),
                cron: "0 0 6,12 * * * *".to_string(),
                condition_n_per_h: Some(5),
                condition_contain: Some(vec!["CLI".to_string()]),
                condition_latest_in_h: Some(1),
                report_json_dir: Some("./default_reports".to_string()),
                report_os_content: Some("Reported matching the condition.".to_string()),
            }
        }
    }

    /// Configファイルの全体
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub struct AllConfig {
        pub search_and_reports: Vec<SearchAndReportConfig>,
    }

    impl Default for AllConfig {
        fn default() -> Self {
            Self {
                search_and_reports: vec![Default::default()],
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::{AllConfig, SearchAndReportConfig, SearchConfig};

        #[tracing_test::traced_test]
        #[test]
        fn test_deserialize() {
            let config_json = r#"
{
    "search_and_reports": [
        {
            "keywords": ["Rust"],
            "cron": "0 0 6 * * * *",
            "condition_n_per_h": 10,
            "condition_contain": ["CLI", "TUI"],
            "report_json_dir": "./my_reports"
        }
    ] 
}
            "#;

            let mut deserializer = serde_json::Deserializer::from_str(&config_json);

            let res: Result<AllConfig, _> = serde_path_to_error::deserialize(&mut deserializer);

            let config = AllConfig {
                search_and_reports: vec![SearchAndReportConfig {
                    search_config: SearchConfig {
                        keywords: vec!["Rust".to_string()],
                    },
                    cron: "0 0 6 * * * *".to_string(),
                    condition_n_per_h: Some(10),
                    condition_contain: Some(vec!["CLI".to_string(), "TUI".to_string()]),
                    condition_latest_in_h: None,
                    report_json_dir: Some("./my_reports".to_string()),
                    report_os_content: None,
                }],
            };

            assert_eq!(res.unwrap(), config);
        }
    }
}

use config::{AllConfig, SearchAndReportConfig};
use search_and_report::{
    predicates::{self, PredListAny},
    reporter::{self, ReporterList},
};

use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::info;

/// アプリケーションのスケジューリングを行う．
async fn schedule_and_run_app(
    config: config::AllConfig,
    instant: bool,
) -> Result<JobScheduler, Box<dyn std::error::Error>> {
    let AllConfig { search_and_reports } = config;

    let scheduler = JobScheduler::new().await?;

    for search_and_report_config in search_and_reports.into_iter() {
        let SearchAndReportConfig {
            search_config,
            cron,
            condition_n_per_h,
            condition_contain,
            condition_latest_in_h,
            report_json_dir,
            report_os_content,
        } = search_and_report_config;

        // Conditionについて
        let mut pred_list = PredListAny::new();
        condition_n_per_h.into_iter().for_each(|condition_n_per_h| {
            let pred = predicates::NumberPerDuration::new(
                condition_n_per_h as usize,
                chrono::Duration::hours(1),
            );
            pred_list.append_pred(move |posts| pred.predicate(posts));
        });
        condition_contain.into_iter().for_each(|condition_contain| {
            let pred = predicates::ContainsKeyWords::new(condition_contain);
            pred_list.append_pred(move |posts| pred.predicate(posts));
        });
        condition_latest_in_h
            .into_iter()
            .for_each(|condition_latest_in_h| {
                let pred = predicates::LatestPostTime::new(chrono::Duration::hours(
                    condition_latest_in_h as i64,
                ));
                pred_list.append_pred(move |posts| pred.predicate(posts));
            });

        // Reportについて
        let mut report_list = ReporterList::new();
        report_json_dir.into_iter().for_each(|report_json_dir| {
            let report = reporter::JsonSaveReporter::new(std::path::Path::new(&report_json_dir));
            report_list.append_reporter(report);
        });
        report_os_content.into_iter().for_each(|report_os_content| {
            let report = reporter::NotificationReporter::new(report_os_content);
            report_list.append_reporter(report);
        });

        // 即時実行
        if instant {
            info!("search and report immediately.");
            search_and_report::search_and_report(
                &search_config,
                search_and_report::platforms::YahooJp,
                &report_list,
                |posts| pred_list.predicate(posts),
            )
            .await?;
        }

        // スケジュール
        let job = Job::new_async(cron.as_str(), {
            let search_config = Arc::new(search_config);
            let report_list = Arc::new(report_list);
            let pred_list = Arc::new(pred_list);

            move |_id, _lock| {
                let search_config = Arc::clone(&search_config);
                let report_list = Arc::clone(&report_list);
                let pred_list = Arc::clone(&pred_list);

                Box::pin(async move {
                    let res = search_and_report::search_and_report(
                        &search_config,
                        search_and_report::platforms::YahooJp,
                        report_list.as_ref(),
                        |posts| pred_list.predicate(posts),
                    )
                    .await;

                    if let Err(e) = res {
                        tracing::error!("Error occurred. {:?}", e);
                    }
                })
            }
        })?;

        scheduler.add(job).await?;
    }

    Ok(scheduler)
}

use clap::Parser;

#[derive(Debug, Parser)]
struct Arg {
    /// config file path.
    #[arg(short, long)]
    config: Option<String>,

    /// search and report immediately
    #[arg(short, long, long, default_value_t = true)]
    instant: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use config::{AllConfig, ConfigError};

    use tracing::info;
    use tracing_subscriber::FmtSubscriber;

    // tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let Arg { config, instant } = Arg::parse();

    /// configファイルの読み取りかデフォルトの作製．
    fn read_or_create_config(
        path: &std::path::Path,
    ) -> Result<AllConfig, Box<dyn std::error::Error>> {
        if path.is_file() {
            // ファイルの場合
            use std::io::Read;

            let mut file = std::fs::File::open(path)?;
            let mut buf = String::new();

            file.read_to_string(&mut buf)?;

            let app_config: AllConfig =
                serde_json::from_str(&buf).map_err(|e| ConfigError(e.to_string()))?;
            Ok(app_config)
        } else {
            // ファイルでない場合．
            use std::io::Write;

            info!("Creating default config file into {:?}.", path);
            let default_config = AllConfig::default();

            let mut file = std::fs::File::create(path)?;
            file.write_all(serde_json::to_string_pretty(&default_config)?.as_bytes())?;

            Ok(default_config)
        }
    }

    // ファイルを開いて読み込み
    let all_config = if let Some(path) = config {
        // パスが与えられていた場合
        read_or_create_config(path.as_ref())?
    } else {
        // パスが与えられていない場合
        let default_path = std::path::Path::new("./default_config.json");
        read_or_create_config(default_path)?
    };

    let scheduler = schedule_and_run_app(all_config, instant).await?;

    info!("scheduler started.");
    scheduler.start().await?;

    // メインスレッドが終了しないように待つ
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(300)).await;
    }
}