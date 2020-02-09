extern crate chrono;
extern crate reqwest;
extern crate serde_json;
extern crate serde;

mod collector;

use collector::{Collector, AdStatus, AdMetric, merge_results};

use chrono::{DateTime, NaiveDateTime, Utc, NaiveDate, NaiveTime};

use std::fs::File;
use std::io::Write;
use std::collections::HashMap;

fn save_results(res: &HashMap<String, AdMetric>, path: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(serde_json::to_string(res).unwrap().as_bytes())?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let start_date = NaiveDate::from_ymd(2020, 1, 1);
    let start_time = NaiveTime::from_hms(0, 0, 0);
    let start_date_time = NaiveDateTime::new(start_date, start_time);

    let end_date = NaiveDate::from_ymd(2020, 2, 9);
    let end_time = NaiveTime::from_hms(0, 0, 0);
    let end_date_time = NaiveDateTime::new(end_date, end_time);

    let collector = Collector {
        start_date_time: DateTime::<Utc>::from_utc(start_date_time, Utc),
        end_date_time: DateTime::<Utc>::from_utc(end_date_time, Utc),
        ad_status: AdStatus::ALL,
        page_ids: vec![153080620724],
        access_token: "ACCESS_TOKEN".to_string(),
        retries: 3,
        batch_size: 1000,
        endpoint: None,
    };

    // Collect data from the Ad Library API
//    let res = collector.collect().await?;
//    save_results(&res, "trump_ad_stats.json");

    // Merge two resulting files together ... can be used to consolidate data collection done over many days
//    let _ = merge_results("trump_ad_stats_merged2.json", "trump_ad_stats2.json", "trump_ad_stats_merged3.json");
    return Ok(());
}
