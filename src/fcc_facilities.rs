use crate::config::Config;
use chrono::{DateTime, Utc};
use futures::lock::Mutex;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use log::info;
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc, time::SystemTime, usize};
use std::{fs::File, io::prelude::*};
use std::{io::BufReader, path::PathBuf};
use tokio::task;
use tokio::time::{sleep, Duration};

// Indexes of data that matters in FCC CSV file
static LIC_EXPIRATION_DATE: usize = 15;
static NIELSEN_DMA: usize = 27;
static FAC_STATUS: usize = 16;
static FAC_SERVICE: usize = 10;
static FAC_CALLSIGN: usize = 5;
static FAC_CHANNEL: usize = 6;
static TV_VIRTUAL_CHANNEL: usize = 28;

static SERVICE_LIST: &'static [&str] = &["DT", "TX", "TV", "TB", "LD", "DC"];

static MAX_FILE_AGE: u64 = 24 * 60 * 60; // 24 hours
static CHECK_INTERVAL: u64 = 60 * 60; // 1 hour

static FACILITIES_URL: &str =
    "https://storage.googleapis.com/locast2tuner/facility.zip";
static DMA_URL: &str = "https://api.locastnet.org/api/dma";

// FCC Facilities are used to map locast stations with FCC channel numbers. After starting the facility,
// the `FacilitiesMap` will contain a mapping from (locast_id, call_sign) to (fac_channel, tv_virtual_channel)
#[derive(Debug)]
pub struct FCCFacilities {
    config: Arc<Config>,
    facilities_map: FacilitiesMap,
}

// (locast_id, call_sign) --> (fac_channel, tv_virtual_channel)
type FacilitiesMap = Arc<Mutex<HashMap<(i64, String), (String, String)>>>;

impl FCCFacilities {
    /// Create a new facilities. Normally this only has to be done once.
    pub async fn new(config: Arc<Config>) -> FCCFacilities {
        // Make sure we have a complete facilities object before returning
        let facilities_map = Arc::new(Mutex::new(
            load(&config.cache_directory.join("facilities")).await,
        ));

        // Start a background thread that will update the facilities periodically
        start_updater_thread(&facilities_map, &config);

        // Build and return
        FCCFacilities {
            config,
            facilities_map,
        }
    }

    /// Look up facilities based on a locast_id (or locast dma), call_sign and potential sub_channel
    pub async fn lookup(&self, locast_dma: i64, call_sign: &str, sub_channel: &str) -> String {
        let facilities_map = self.facilities_map.lock().await;
        let (fac_channel, tv_virtual_channel) = facilities_map
            .get(&(locast_dma, call_sign.to_string()))
            .unwrap(); // This should exist

        if tv_virtual_channel.is_empty() {
            fac_channel.to_owned()
        } else if sub_channel.is_empty() {
            format!("{}.1", fac_channel.as_str()) // default to x.1 if there is no sub_channel
        } else {
            format!("{}.{}", fac_channel.as_str(), sub_channel)
        }
    }
}

/// Start an thread that will update the facilities map regularly and store them
/// in the cache directory
fn start_updater_thread(facilities_map: &FacilitiesMap, config: &Arc<Config>) {
    let facilities_map = facilities_map.clone();
    let config = config.clone();

    task::spawn(async move {
        loop {
            sleep(Duration::from_secs(CHECK_INTERVAL)).await;

            info!("Reloading FCC facilities..");
            let cache_file = config.cache_directory.join("facilities");
            let new_facilties = load(&cache_file).await;
            let mut facilities = facilities_map.lock().await;
            *facilities = new_facilties;
        }
    });
}

/// Check if a path has expired, based on `MAX_FILE_AGE`
fn path_expired(path: &PathBuf) -> bool {
    let modified = path.metadata().unwrap().modified().unwrap();
    SystemTime::now()
        .duration_since(modified)
        .unwrap()
        .as_secs()
        > MAX_FILE_AGE
}

/// Load facilities from `cache_file`
async fn load<'a>(cache_file: &PathBuf) -> HashMap<(i64, String), (String, String)> {
    // First get the locast_dmas from locast.org
    let locast_dmas: Vec<LocastDMA> = crate::utils::get(DMA_URL, None, 100)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let lines: Vec<Result<String, std::io::Error>>;
    // Using cached facilities if possible.
    let downloaded = if cache_file.exists() && !path_expired(&cache_file) {
        info!("Using cached FCC facilities at {}", cache_file.display());
        lines = BufReader::new(File::open(cache_file).unwrap())
            .lines()
            .collect::<Vec<Result<String, std::io::Error>>>();
        false
    } else {
        info!("Downloading FCC facilities");
        let zipfile = crate::utils::get(FACILITIES_URL, None, 100)
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();

        lines = BufReader::new(
            zip::ZipArchive::new(std::io::Cursor::new(zipfile))
                .unwrap()
                .by_name("facility.dat")
                .unwrap(),
        )
        .lines()
        .collect::<Vec<Result<String, std::io::Error>>>();
        true
    };

    let mut loaded_lines: Vec<String> = Vec::new();
    let mut facilities_map: HashMap<(i64, String), (String, String)> = HashMap::new();
    for line in lines.into_iter().map(|l| l.unwrap()) {
        let parts: Vec<&str> = line.split("|").collect();

        let lic_expiration_date = parts[LIC_EXPIRATION_DATE];
        let nielsen_dma = parts[NIELSEN_DMA];
        let fac_status = parts[FAC_STATUS];
        let fac_service = parts[FAC_SERVICE];
        let fac_call_sign = parts[FAC_CALLSIGN];

        // Used for lookup
        let fac_channel = &parts[FAC_CHANNEL];
        let tv_virtual_channel = &parts[TV_VIRTUAL_CHANNEL];

        if fac_status == "LICEN"
            && lic_expiration_date != ""
            && nielsen_dma != ""
            && SERVICE_LIST.contains(&fac_service)
        {
            let s = format!("{} 23:59:59 +0000", lic_expiration_date);
            if DateTime::parse_from_str(&s, "%m/%d/%Y %T %z").unwrap() >= Utc::now() {
                let call_sign = fac_call_sign.split("-").collect::<Vec<&str>>()[0];

                // Get the locast_id based on the Nielsen DMA
                let locast_id = nielsen_dma_to_locast_id(nielsen_dma, &locast_dmas);
                if locast_id.is_some() {
                    facilities_map.insert(
                        (locast_id.unwrap(), call_sign.to_owned()),
                        (fac_channel.to_string(), tv_virtual_channel.to_string()),
                    );
                    loaded_lines.push(line);
                }
            }
        }
    }

    // Only write lines that matter to the cache file.
    if downloaded {
        write_cache_file(cache_file, loaded_lines.join("\n").as_bytes());
    }

    facilities_map
}

/// Try to find a locast_id by matching a Nielsen DMA with a Locast DMA name. This uses a fuzzy matcher.
fn nielsen_dma_to_locast_id(nielsen_dma: &str, locast_dmas: &Vec<LocastDMA>) -> Option<i64> {
    let matcher = SkimMatcherV2::default();
    let mut matches: Vec<(i64, i64)> = locast_dmas
        .iter()
        .map(|l| (l.id, l.name.to_lowercase()))
        .map(|(locast_id, name)| (locast_id, matcher.fuzzy_match(&nielsen_dma, &name)))
        .filter(|(_, ratio)| ratio.is_some() && ratio.unwrap() > 115)
        .map(|(locast_id, ratio)| (locast_id, ratio.unwrap()))
        .collect();

    matches.sort_by(|a, b| b.1.cmp(&a.1)); // Reversed sort
    Some(matches.first()?.0)
}

/// Write the cache file to `cache_path`
fn write_cache_file(cache_file: &PathBuf, contents: &[u8]) {
    let display = cache_file.display();
    let mut file = match File::create(&cache_file) {
        Err(why) => panic!("Couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(contents) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => info!("Cached FCC facilities to {}", display),
    }
}

/// Struct used to deserialize Locast DMA json
#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct LocastDMA {
    id: i64,
    name: String,
}
