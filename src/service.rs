use crate::{config::Config, credentials::LocastCredentials, fcc_facilities::FCCFacilities};
use chrono::Utc;
use regex::Regex;
use serde::Deserialize;
use std::fmt;
use std::{convert::From, str::FromStr};

static DMA_URL: &str = "https://api.locastnet.org/api/watch/dma";
static IP_URL: &str = "https://api.locastnet.org/api/watch/dma/ip";
static STATIONS_URL: &str = "https://api.locastnet.org/api/watch/epg";

#[derive(Debug)]
pub struct LocastService<'a, 'b, 'c> {
    config: &'a Config,
    credentials: &'b LocastCredentials<'b>,
    fcc_facilities: &'c FCCFacilities<'c>,
    zipcode: Option<String>,
    geo: Geo,
    uuid: String,
}

impl<'a, 'b, 'c> LocastService<'a, 'b, 'c> {
    pub fn new(
        config: &'a Config,
        credentials: &'b LocastCredentials<'b>,
        fcc_facilities: &'c FCCFacilities<'c>,
        zipcode: Option<String>,
    ) -> LocastService<'a, 'b, 'c> {
        let geo = Geo::from(&zipcode);
        if !geo.active {
            panic!(format!("{} not active", geo.name))
        }

        let uuid = uuid::Uuid::new_v5(
            &uuid::Uuid::from_str(&config.uuid).unwrap(),
            geo.DMA.as_bytes(),
        )
        .to_string();

        LocastService {
            config,
            credentials,
            fcc_facilities,
            zipcode,
            geo: geo,
            uuid,
        }
    }

    // TODO: Caching
    pub fn stations(&self) {
        self.build_stations();
    }

    fn build_stations(&self) {
        let locast_stations = self.locast_stations();

        for mut station in locast_stations.into_iter() {
            station.timezone = self.geo.timezone.to_owned();
            station.city = Some(self.geo.name.to_owned());

            let channel_from_call_sign = match Regex::new(r"(\d+\.\d+) .+")
                .unwrap()
                .captures(&station.callSign)
            {
                Some(c) => Some(c.get(1).map_or("", |m| m.as_str())),
                None => None,
            };

            let c = if let Some(channel) = channel_from_call_sign {
                Some(channel.to_string())
            } else if let Some((call_sign, sub_channel)) =
                detect_callsign(&station.name).or(detect_callsign(&station.callSign))
            {
                let dma = self.geo.DMA.parse::<i64>().unwrap();
                let (fcc_channel, analog) = self
                    .fcc_facilities
                    .by_dma_and_call_sign(&(dma, call_sign.to_string()));
                let channel = if let true = analog {
                    fcc_channel.to_string()
                } else if sub_channel.is_empty() {
                    format!("{}.1", fcc_channel)
                } else {
                        format!("{}.{}", fcc_channel, sub_channel)
                };
                Some(channel.to_string())
            } else {
                panic!(format!(
                    "Channel {}, call sign: {} not found!",
                    &station.name, &station.callSign
                ));
            };
            station.channel = c;
        }
    }

    fn locast_stations(&self) -> Vec<Station> {
        let start_time = Utc::now().format("%Y-%m-%dT00:00:00-00:00").to_string();
        let uri = format!(
            "{}/{}?startTime={}&hours={}",
            STATIONS_URL,
            self.geo.DMA,
            start_time,
            self.config.days * 24
        );
        crate::utils::get(&uri, Some(&self.credentials.token()))
            .json::<Vec<Station>>()
            .unwrap()
    }
}

impl<'a, 'b, 'c> fmt::Display for LocastService<'a, 'b, 'c> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "LocastService{{ zipcode: {:?}, uuid: {:?}, geo: {:?} }}",
            self.zipcode, self.uuid, self.geo
        )
    }
}

fn detect_callsign(input: &str) -> Option<(&str, &str)> {
    let re = Regex::new(r"^([KW][A-Z]{2,3})[A-Z]{0,2}(\d{0,2})$").unwrap();
    let caps = re.captures(input)?;
    let call_sign = caps.get(1).map_or("default", |m| m.as_str());
    let sub_channel = caps.get(2).map_or("default", |m| m.as_str());
    Some((call_sign, sub_channel))
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Geo {
    latitude: f64,
    longitude: f64,
    DMA: String,
    name: String,
    active: bool,
    timezone: Option<String>,
}

impl From<&Option<String>> for Geo {
    fn from(zipcode: &Option<String>) -> Self {
        let uri = match zipcode {
            Some(z) => format!("{}/zip/{}", DMA_URL, z),
            None => String::from(IP_URL),
        };

        let mut geo = crate::utils::get(&uri, None).json::<Geo>().unwrap();
        geo.timezone = Some(tz_search::lookup(geo.latitude, geo.longitude).unwrap());
        geo
    }
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Station {
    id: i64,
    dma: i64,
    stationId: String,
    name: String,
    callSign: String,
    logoUrl: String,
    active: bool,
    listings: Vec<Listing>,
    timezone: Option<String>,
    city: Option<String>,
    channel: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Listing {
    airdate: Option<i64>,
    audioProperties: Option<String>,
    description: Option<String>,
    duration: i32,
    entityType: String,
    episodeNumber: Option<i16>,
    episodeTitle: Option<String>,
    genres: Option<String>,
    hasImageArtwork: bool,
    hasSeriesArtwork: bool,
    isNew: Option<bool>,
    preferredImage: Option<String>,
    preferredImageHeight: Option<i16>,
    preferredImageWidth: Option<i16>,
    programId: String,
    rating: Option<String>,
    releaseDate: Option<i64>,
    releaseYear: Option<i16>,
    seasonNumber: Option<i16>,
    seriesId: Option<String>,
    shortDescription: Option<String>,
    showType: String,
    startTime: i64,
    stationId: i64,
    title: String,
    topCast: Option<String>,
    videoProperties: Option<String>,
}
