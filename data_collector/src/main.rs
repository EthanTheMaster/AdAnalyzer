extern crate chrono;
extern crate reqwest;
extern crate serde_json;
extern crate serde;
extern crate clap;

mod collector;

use collector::{Collector, AdStatus, AdMetric, merge_results};

use chrono::{DateTime, NaiveDateTime, Utc, NaiveDate, NaiveTime};

use clap::{Arg, App, SubCommand, ArgMatches};

use std::fs::{File, DirBuilder};
use std::path::PathBuf;
use std::io::Write;
use std::collections::HashMap;

fn save_results(res: &HashMap<String, AdMetric>, path_dir: &str) -> std::io::Result<()> {
    // Create directory if it does not exist
    DirBuilder::new().recursive(true).create(path_dir)?;
    let data_location = PathBuf::from(path_dir).join("ad_data.json");

    let mut file = File::create(data_location.as_path())?;
    file.write_all(serde_json::to_string(res).unwrap().as_bytes())?;
    Ok(())
}

async fn parse_collect_subcommand(matches: &ArgMatches<'_>) -> Result<(), String> {
    // Unwrap all arguments that are either required or have some default value
    let save_path = matches.value_of("save_path").unwrap();
    let access_token = matches.value_of("access_token").unwrap();
    let page_ids = matches.values_of("page_ids")
                                                                .unwrap()
                                                                .map(|id_str| id_str.parse::<u64>());
    if !page_ids.clone().all(|r| r.is_ok()) {
        return Err("Page ids must be 64 bit (unsigned) integers".to_string());
    }
    let page_ids: Vec<u64> = page_ids.map(|r| r.unwrap()).collect();

    let year_start = matches.value_of("year_start").unwrap().parse::<i32>().map_err(|_| "Failed to parse year_start")?;
    let month_start = matches.value_of("month_start").unwrap().parse::<u32>().map_err(|_| "Failed to parse month_start")?;
    let day_start = matches.value_of("day_start").unwrap().parse::<u32>().map_err(|_| "Failed to parse day_start")?;

    let year_end = matches.value_of("year_end").unwrap().parse::<i32>().map_err(|_| "Failed to parse year_end")?;
    let month_end = matches.value_of("month_end").unwrap().parse::<u32>().map_err(|_| "Failed to parse month_end")?;
    let day_end = matches.value_of("day_end").unwrap().parse::<u32>().map_err(|_| "Failed to parse day_end")?;

    let retries = matches.value_of("retries").unwrap().parse::<usize>().map_err(|_| "Failed to parse retries")?;
    let batch_size = matches.value_of("batch_size").unwrap().parse::<usize>().map_err(|_| "Failed to parse batch_size")?;
    let endpoint = matches.value_of("endpoint").map(|s| String::from(s));
    let ad_status = matches.value_of("ad_status").unwrap();

    // Convert ad_status string to enum
    let ad_status: AdStatus = match ad_status.to_uppercase().as_str() {
        "ALL" => {Ok(AdStatus::ALL)},
        "ACTIVE" => {Ok(AdStatus::ACTIVE)},
        "INACTIVE" => {Ok(AdStatus::INACTIVE)}
        _ => {Err("Invalid value for ad_status")}
    }?;

    // Create date time and validate dates
    let start_date = NaiveDate::from_ymd_opt(year_start, month_start, day_start)
                                    .ok_or(format!("Invalid starting date: {}-{}-{}", year_start, month_start, day_start))?;
    let start_time = NaiveTime::from_hms(0, 0, 0);
    let start_date_time = NaiveDateTime::new(start_date, start_time);

    let end_date = NaiveDate::from_ymd_opt(year_end, month_end, day_end)
                                    .ok_or(format!("Invalid ending date: {}-{}-{}", year_end, month_end, day_end))?;
    let end_time = NaiveTime::from_hms(23, 59, 59);
    let end_date_time = NaiveDateTime::new(end_date, end_time);

    if end_date_time < start_date_time {
        return Err("Starting date must be before the ending date".to_string());
    }

    let collector = Collector {
        start_date_time: DateTime::<Utc>::from_utc(start_date_time, Utc),
        end_date_time: DateTime::<Utc>::from_utc(end_date_time, Utc),
        ad_status,
        page_ids,
        access_token: String::from(access_token),
        retries,
        batch_size,
        endpoint,
    };

    // Collect data from the Ad Library API
    let res = collector.collect().await.map_err(|e| e.to_string())?;
    save_results(&res, save_path).map_err(|e| e.to_string()).map_err(|_| "Failed to save results")?;

    return Ok(());
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let matches = App::new("Collects ads from Facebook's Ad Library API and runs web interface to do analysis.")
                        .subcommand(SubCommand::with_name("collect")
                            .about("Collects ads from Facebook's Ad Library API")
                            .arg(Arg::with_name("save_path")
                                .long("save_path")
                                .required(true)
                                .help("Directory to save the collected data")
                                .takes_value(true)
                            )
                            .arg(Arg::with_name("access_token")
                                .long("access_token")
                                .required(true)
                                .help("Access token for Ad Library API")
                                .takes_value(true)
                            )
                            .arg(Arg::with_name("page_ids")
                                .long("page_ids")
                                .required(true)
                                .help("Facebook page ids to grab ads from")
                                .takes_value(true)
                                .use_delimiter(true)
                            )
                            .arg(Arg::with_name("year_start")
                                .long("year_start")
                                .required(true)
                                .help("Starting date's year")
                                .takes_value(true)
                            )
                            .arg(Arg::with_name("month_start")
                                .long("month_start")
                                .required(false)
                                .help("Starting date's month (1-12)")
                                .takes_value(true)
                                .default_value("1")
                            )
                            .arg(Arg::with_name("day_start")
                                .long("day_start")
                                .required(false)
                                .help("Starting date's day (1-31)")
                                .takes_value(true)
                                .default_value("1")
                            )
                            .arg(Arg::with_name("year_end")
                                .long("year_end")
                                .required(true)
                                .help("Ending date's year")
                                .takes_value(true)
                            )
                            .arg(Arg::with_name("month_end")
                                .long("month_end")
                                .required(false)
                                .help("Ending date's month (1-12)")
                                .takes_value(true)
                                .default_value("12")
                            )
                            .arg(Arg::with_name("day_end")
                                .long("day_end")
                                .required(false)
                                .help("Ending date's day (1-31)")
                                .takes_value(true)
                                .default_value("31")
                            )
                            .arg(Arg::with_name("retries")
                                .long("retries")
                                .required(false)
                                .help("Number of times to retry request to API before quitting")
                                .takes_value(true)
                                .default_value("3")
                            )
                            .arg(Arg::with_name("batch_size")
                                .long("batch_size")
                                .required(false)
                                .help("Number of ads that should be returned by API at a time")
                                .takes_value(true)
                                .default_value("1000")
                            )
                            .arg(Arg::with_name("endpoint")
                                .long("endpoint")
                                .required(false)
                                .help("Custom API endpoint is used for collecting ads. Can be used to resume progress.")
                                .takes_value(true)
                            )
                            .arg(Arg::with_name("ad_status")
                                .long("ad_status")
                                .required(false)
                                .help("Filter ads by status: ALL, ACTIVE, or INACTIVE")
                                .takes_value(true)
                                .default_value("ALL")
                                .possible_values(&["ALL", "ACTIVE", "INACTIVE"])
                                .case_insensitive(true)
                            )
                        )
                        .subcommand(SubCommand::with_name("merge")
                            .about("Merges two jsons files that were generated during collection")
                            .arg(Arg::with_name("path1")
                                .required(true)
                                .takes_value(true)
                            )
                            .arg(Arg::with_name("path2")
                                .required(true)
                                .takes_value(true)
                            )
                            .arg(Arg::with_name("target")
                                .required(true)
                                .takes_value(true)
                            )
                        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("collect") {
        parse_collect_subcommand(matches).await?;
    }else if let Some(matches) = matches.subcommand_matches("merge") {
        // Merge two resulting files together ... can be used to consolidate data collection done over many days
        merge_results(
            matches.value_of("path1").unwrap(),
            matches.value_of("path2").unwrap(),
            matches.value_of("target").unwrap()
        ).map_err(|_| "Failed to merge files")?;
    }

    return Ok(());
}
