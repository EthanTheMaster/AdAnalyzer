use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

use std::fmt;
use std::fmt::{Formatter, Error};
use std::collections::HashMap;
use std::str::FromStr;
use std::f64;
use std::fs::{File, DirBuilder};
use std::path::PathBuf;
use std::io::{Read, Write};

pub enum AdStatus {
    ALL,
    ACTIVE,
    INACTIVE,
}

impl fmt::Display for AdStatus {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            AdStatus::ALL => {
                write!(f, "ALL")
            },
            AdStatus::ACTIVE => {
                write!(f, "ACTIVE")
            },
            AdStatus::INACTIVE => {
                write!(f, "INACTIVE")
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiDemographic {
    age: String,
    gender: String,
    percentage: String
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiCountRange {
    lower_bound: Option<String>,
    upper_bound: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiRegion {
    region: String,
    percentage: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiAdData {
    ad_creative_body: Option<String>,
    ad_delivery_start_time: String,
    ad_delivery_stop_time: Option<String>,
    demographic_distribution: Option<Vec<ApiDemographic>>,
    impressions: ApiCountRange,
    region_distribution: Option<Vec<ApiRegion>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Cursor {
    next: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    data: Vec<ApiAdData>,
    paging: Cursor,
}

pub struct Collector {
    // Obtain ads that were posted within time frame
    pub start_date_time: DateTime<Utc>,
    pub end_date_time: DateTime<Utc>,
    pub ad_status: AdStatus,
    pub page_ids: Vec<u64>,
    pub access_token: String,
    pub retries: usize,
    pub batch_size: usize,
    // Start at user-provided endpoint ... may be used to continue progress after failure
    pub endpoint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdMetric {
    // Maps demographic (gender and age) to raw impression count (lower and upper bound)
    pub demographic_impression: HashMap<String, (f64, f64)>,
    // Maps region to raw impression count (lower and upper bound)
    pub region_impression: HashMap<String, (f64, f64)>
}


impl Collector {
    pub async fn collect(&self) -> Result<HashMap<String, AdMetric>, reqwest::Error> {
        let client = reqwest::Client::new();
        let mut endpoint: String =
            match &self.endpoint {
                None => {
                    format!(
                        "https://graph.facebook.com/v5.0/ads_archive?\
                        fields=ad_creative_body,ad_delivery_start_time,ad_delivery_stop_time,demographic_distribution,impressions,region_distribution,spend&\
                        ad_type=POLITICAL_AND_ISSUE_ADS&ad_reached_countries=['US']&ad_active_status={}&search_page_ids={:?}&limit={}&access_token={}",
                        self.ad_status, self.page_ids, self.batch_size, self.access_token
                    )
                },
                Some(value) => {
                    value.clone()
                },
            };

        let mut res: HashMap<String, AdMetric> = HashMap::new();
        let mut retries: usize = 0;
        loop {
            println!("-------------------------------------");
            println!("Endpoint: {}", endpoint);
            let api_response_content: String = client.get(endpoint.as_str()).send().await?.text().await?;
            let api_response_result: Result<ApiResponse, serde_json::Error> = serde_json::from_str(&api_response_content);
            match api_response_result {
                Ok(api_response) => {
                    // Reset retry counter
                    retries = 0;
                    if !api_response.data.is_empty() {
                        println!("From {} to {}", &api_response.data[0].ad_delivery_start_time, &api_response.data.last().unwrap().ad_delivery_start_time);
                    }
                    for ad in api_response.data.iter() {
                        // let ad_start: DateTime<Utc> = DateTime::from(DateTime::parse_from_str(ad.ad_delivery_start_time.as_str(), "%Y-%m-%dT%H:%M:%S%z").unwrap());
                        // Facebook changed api format ... normalize date to midnight UTC time
                        let ad_start: DateTime<Utc> = DateTime::from(DateTime::parse_from_str(&*(ad.ad_delivery_start_time.as_str().to_owned() + "T00:00:00+0000"), "%Y-%m-%dT%H:%M:%S%z").unwrap());
                        // Consider only ads that started within specified time frame
                        if ad_start < self.start_date_time {
                            return Ok(res);
                        }
                        if ad_start > self.end_date_time {
                            continue;
                        }
                        // Ignore ads with no bodies
                        if ad.ad_creative_body.is_none() {
                            continue;
                        }

                        // If the ad has never been seen before, add it to map
                        let ad_body = ad.ad_creative_body.as_ref().unwrap().clone();
                        if !res.contains_key(&ad_body) {
                            res.insert(ad_body.clone(), AdMetric { demographic_impression: HashMap::new(), region_impression: HashMap::new() });
                        }

                        let metric: &mut AdMetric = res.get_mut(&ad_body).unwrap();
                        // Update demographic impression count
                        let impression_lower_bound = f64::from_str(ad.impressions.lower_bound.as_ref().unwrap_or(&format!("0.0"))).unwrap();
                        let impression_upper_bound = f64::from_str(ad.impressions.upper_bound.as_ref().unwrap_or(&format!("{}", impression_lower_bound))).unwrap();
                        if ad.demographic_distribution.is_some() {
                            for demographic in ad.demographic_distribution.as_ref().unwrap().iter() {
                                // In order to later serialize the resulting HashMap, the demographic_key needs to be a String
                                let demographic_key = demographic.gender.clone() + "/" + demographic.age.as_str();
                                let demographic_percentage = f64::from_str(&demographic.percentage).unwrap();
                                // If demographic has never been seen before, add it to map
                                if !metric.demographic_impression.contains_key(&demographic_key) {
                                    metric.demographic_impression.insert(demographic_key.clone(), (0.0, 0.0));
                                }

                                let (prev_lower, prev_upper) = metric.demographic_impression.get(&demographic_key).cloned().unwrap();
                                metric.demographic_impression.insert(
                                    demographic_key.clone(),
                                    (prev_lower + impression_lower_bound * demographic_percentage, prev_upper + impression_upper_bound * demographic_percentage)
                                );
                            }
                        }
                        if ad.region_distribution.is_some() {
                            for region in ad.region_distribution.as_ref().unwrap().iter() {
                                let region_key = region.region.clone();
                                let region_percentage = f64::from_str(&region.percentage).unwrap();
                                // If region has never been seen before, add it to map
                                if !metric.region_impression.contains_key(&region_key) {
                                    metric.region_impression.insert(region_key.clone(), (0.0, 0.0));
                                }

                                let (prev_lower, prev_upper) = metric.region_impression.get(&region_key).cloned().unwrap();
                                metric.region_impression.insert(
                                    region_key.clone(),
                                    (prev_lower + impression_lower_bound * region_percentage, prev_upper + impression_upper_bound * region_percentage)
                                );
                            }
                        }
                    }


                    if let Some(next_endpoint) = api_response.paging.next {
                        endpoint = next_endpoint;
                    } else {
                        // Reached end of data
                        break;
                    }
                },
                Err(e) => {
                    if retries >= self.retries {
                        break;
                    }
                    retries += 1;
                    println!("Failed ... Retrying({}/{})", retries, self.retries);
                    println!("Error: {:?}", e);
                },
            }
        }

        return Ok(res);
    }
}

// Merge two serialized results
// Note: If file1 and file2 tabulated results from ads present in both datasets ... the merged
//       result will be incorrect as there will be double counting.
pub fn merge_results(path1: &str, path2: &str, target_path: &str) -> std::io::Result<()> {
    let mut file1 = File::open(path1)?;
    let mut file2 = File::open(path2)?;

    let mut file1_content = String::new();
    let mut file2_content = String::new();

    file1.read_to_string(&mut file1_content)?;
    file2.read_to_string(&mut file2_content)?;

    let doc1: HashMap<String, AdMetric> = serde_json::from_str(&file1_content).unwrap();
    let doc2: HashMap<String, AdMetric> = serde_json::from_str(&file2_content).unwrap();
    let mut res: HashMap<String, AdMetric> = HashMap::new();

    // Merge both documents
    for ad_message in doc1.keys() {
        res.insert(ad_message.clone(), doc1.get(ad_message).cloned().unwrap());
    }
    for ad_message in doc2.keys() {
        if res.contains_key(ad_message) {
            // Ad was in doc1 ... combine
            let doc2_metrics = doc2.get(ad_message).unwrap();
            let res_metrics = res.get_mut(ad_message).unwrap();
            // Merge demographic metrics
            for demographic in doc2_metrics.demographic_impression.keys() {
                let doc2_demographic_metric = *doc2_metrics.demographic_impression.get(demographic).unwrap();
                if res_metrics.demographic_impression.contains_key(demographic) {
                    // Demographic present in both doc1 and doc2 ... take sum
                    let doc1_demographic_metric = *res_metrics.demographic_impression.get(demographic).unwrap();
                    *res_metrics.demographic_impression.get_mut(demographic).unwrap() = (doc1_demographic_metric.0 + doc2_demographic_metric.0, doc1_demographic_metric.1 + doc2_demographic_metric.1);
                } else {
                    res_metrics.demographic_impression.insert(demographic.clone(), doc2_demographic_metric);
                }
            }
            // Merge region metrics
            for region in doc2_metrics.region_impression.keys() {
                let doc2_region_metric = *doc2_metrics.region_impression.get(region).unwrap();
                if res_metrics.region_impression.contains_key(region) {
                    // Region present in both doc1 and doc2 ... take sum
                    let doc1_region_metric = *res_metrics.region_impression.get(region).unwrap();
                    *res_metrics.region_impression.get_mut(region).unwrap() = (doc1_region_metric.0 + doc2_region_metric.0, doc1_region_metric.1 + doc2_region_metric.1);
                } else {
                    res_metrics.region_impression.insert(region.clone(), doc2_region_metric);
                }
            }
        } else {
            res.insert(ad_message.clone(), doc2.get(ad_message).cloned().unwrap());
        }
    }

    let target_path = PathBuf::from(target_path);
    // Create parent directory if it does not exist
    DirBuilder::new().recursive(true).create(target_path.parent().unwrap())?;

    let mut output = File::create(target_path.as_path())?;
    output.write_all(serde_json::to_string(&res).unwrap().as_bytes())?;

    Ok(())
}

// Saves results from Collector::collect() into json file in path_dir
pub fn save_results(res: &HashMap<String, AdMetric>, path_dir: &str) -> std::io::Result<()> {
    // Create directory if it does not exist
    DirBuilder::new().recursive(true).create(path_dir)?;
    let data_location = PathBuf::from(path_dir).join("ad_data.json");

    let mut file = File::create(data_location.as_path())?;
    file.write_all(serde_json::to_string(res).unwrap().as_bytes())?;
    Ok(())
}