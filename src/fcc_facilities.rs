use crate::config::Config;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::{self, SystemTime},
    usize,
};
use std::{fs::File, io::prelude::*};
use std::{io::BufReader, path::PathBuf};

static LIC_EXPIRATION_DATE: usize = 15;
static NIELSEN_DMA: usize = 27;
static FAC_STATUS: usize = 16;
static FAC_SERVICE: usize = 10;
static FAC_CALLSIGN: usize = 5;
static FAC_CHANNEL: usize = 6;
static TV_VIRTUAL_CHANNEL: usize = 28;

static SERVICE_LIST: &'static [&str] = &["DT", "TX", "TV", "TB", "LD", "DC"];

static MAX_FILE_AGE: u64 = 26 * 60 * 60;
static CHECK_INTERVAL: u64 = 3600;

static FACILITIES_URL: &str =
    "https://transition.fcc.gov/ftp/Bureaus/MB/Databases/cdbs/facility.zip";
static DMA_URL: &str = "http://api.locastnet.org/api/dma";

#[derive(Debug)]
pub struct FCCFacilities {
    config: Arc<Config>,
    facilities_map: FacilitiesMap,
}

// (locast_id, call_sign) --> (fac_channel, tv_virtual_channel)
type FacilitiesMap = Arc<Mutex<HashMap<(i64, String), (String, String)>>>;

impl FCCFacilities {
    pub fn new(config: Arc<Config>) -> FCCFacilities {
        // Make sure we have a complete facilities object before returning
        let facilities_map = Arc::new(Mutex::new(load(&config.cache_directory.join("facilities"))));
        start_updater_thread(&facilities_map, &config);

        let facilities = FCCFacilities {
            config,
            facilities_map,
        };

        // Start reload thread. This runs in the background forever.

        facilities
    }

    pub fn lookup(&self, locast_dma: i64, call_sign: &str, sub_channel: &str) -> String {
        let facilities_map = self.facilities_map.lock().unwrap();
        let (fac_channel, tv_virtual_channel) = facilities_map
            .get(&(locast_dma, call_sign.to_string()))
            .unwrap(); // This should exist

        if tv_virtual_channel.is_empty() {
            fac_channel.to_owned()
        } else if sub_channel.is_empty() {
            let s = format!("{}.1", fac_channel.as_str());
            s
        } else {
            let s = format!("{}.{}", fac_channel.as_str(), sub_channel);
            s
        }
    }
}

fn start_updater_thread(facilities_map: &FacilitiesMap, config: &Arc<Config>) {
    let facilities_map = facilities_map.clone();
    let config = config.clone();

    thread::spawn(move || loop {
        thread::sleep(time::Duration::from_secs(CHECK_INTERVAL));
        println!("Reloading FCC facilities..");
        let cache_file = config.cache_directory.join("facilities");
        let new_facilties = load(&cache_file);
        let mut facilities = facilities_map.lock().unwrap();
        *facilities = new_facilties;
    });
}

fn path_expired(path: &PathBuf) -> bool {
    let modified = path.metadata().unwrap().modified().unwrap();
    SystemTime::now()
        .duration_since(modified)
        .unwrap()
        .as_secs()
        > MAX_FILE_AGE
}

fn load<'a>(cache_file: &PathBuf) -> HashMap<(i64, String), (String, String)> {
    let mut zip: zip::ZipArchive<std::io::Cursor<Bytes>>;
    let reader: Box<dyn Read>;

    let locast_dmas: Vec<LocastDMA> = crate::utils::get(DMA_URL, None).json().unwrap();

    let downloaded = if cache_file.exists() && !path_expired(&cache_file) {
        println!("Using cached FCC facilities at {}", cache_file.display());
        reader = Box::new(File::open(cache_file).unwrap());
        false
    } else {
        println!("Downloading FCC facilities");
        let zipfile = crate::utils::get(FACILITIES_URL, None).bytes().unwrap();
        zip = zip::ZipArchive::new(std::io::Cursor::new(zipfile)).unwrap();
        reader = Box::new(zip.by_name("facility.dat").unwrap());
        true
    };

    let mut facilities_map: HashMap<(i64, String), (String, String)> = HashMap::new();
    let mut loaded_lines: Vec<String> = Vec::new();

    for line in BufReader::new(reader).lines().map(|l| l.unwrap()) {
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

    if downloaded {
        write_cache_file(cache_file, loaded_lines.join("\n").as_bytes());
    }

    facilities_map
}

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

fn write_cache_file(cache_file: &PathBuf, contents: &[u8]) {
    let display = cache_file.display();
    let mut file = match File::create(&cache_file) {
        Err(why) => panic!("Couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(contents) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("Cached FCC facilities to {}", display),
    }
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct LocastDMA {
    id: i64,
    name: String,
}
