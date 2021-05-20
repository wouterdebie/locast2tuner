pub mod multiplexer;
pub mod station;
pub mod stationprovider;
use self::{
    station::{Station, Stations},
    stationprovider::StationProvider,
};
use crate::{
    config::Config, credentials::LocastCredentials, errors::AppError,
    fcc_facilities::FCCFacilities, utils::get,
};
use async_trait::async_trait;
use chrono::Utc;
use futures::lock::Mutex;
use log::info;
use regex::Regex;
use reqwest::Url;
use serde::Deserialize;
use serde_json::Value;
use std::{
    borrow::Cow,
    collections::HashMap,
    convert::{From, TryFrom},
    fmt,
    str::FromStr,
    sync::Arc,
};
use tokio::task;
use tokio::time::{sleep, Duration};

static DMA_URL: &str = "https://api.locastnet.org/api/watch/dma";
static IP_URL: &str = "https://api.locastnet.org/api/watch/dma/ip";
static STATIONS_URL: &str = "https://api.locastnet.org/api/watch/epg";
static WATCH_URL: &str = "https://api.locastnet.org/api/watch/station";

/// Struct that interacts with locast. Note that valid credentials are required
#[derive(Debug)]
pub struct LocastService {
    config: Arc<Config>,
    credentials: Arc<LocastCredentials>,
    fcc_facilities: Arc<FCCFacilities>,
    pub zipcode: Option<String>,
    pub geo: Arc<Geo>,
    pub uuid: String,
    stations: Stations,
}

impl LocastService {
    /// Construct a new LocastService for a specific DMA
    pub async fn new(
        config: Arc<Config>,
        credentials: Arc<LocastCredentials>,
        fcc_facilities: Arc<FCCFacilities>,
        zipcode: Option<String>,
    ) -> LocastServiceArc {
        // Figure out what location we are serving
        let geo = Arc::new(geo_from(&zipcode).await);
        if !geo.active {
            panic!("{} not active", geo.name)
        }

        // Generate a UUID for this specific service
        let uuid = uuid::Uuid::new_v5(
            &uuid::Uuid::from_str(&config.uuid).unwrap(),
            geo.DMA.as_bytes(),
        )
        .to_string();

        // Get a list of stations
        let stations = Arc::new(Mutex::new(
            build_stations(
                locast_stations(&geo.DMA, config.days, &credentials.token().await).await,
                &geo,
                &config,
                &fcc_facilities,
            )
            .await,
        ));

        // Start an updater thread that will periodically update all station information
        // including EPG data
        start_updater_thread(&config, &stations, &geo, &credentials, &fcc_facilities);

        Arc::new(LocastService {
            config,
            credentials,
            fcc_facilities,
            zipcode,
            geo,
            uuid,
            stations,
        })
    }

    /// Convenience method for building stations based on &self
    async fn build_stations(&self) -> Vec<Station> {
        let locast_stations = locast_stations(
            &self.geo.DMA,
            self.config.days,
            &self.credentials.token().await,
        )
        .await;
        build_stations(
            locast_stations,
            &self.geo,
            &self.config,
            &self.fcc_facilities,
        )
        .await
    }
}

pub type LocastServiceArc = Arc<LocastService>;

#[async_trait]
impl StationProvider for LocastServiceArc {
    /// Get stations
    async fn stations(&self) -> Stations {
        if self.config.disable_station_cache {
            Arc::new(Mutex::new(self.build_stations().await))
        } else {
            self.stations.clone()
        }
    }

    /// Get the stream URI for a specified station id
    async fn station_stream_uri(&self, id: &str) -> Result<Mutex<String>, AppError> {
        // Construct the URL for the station
        let url = format!(
            "{}/{}/{}/{}",
            WATCH_URL, id, self.geo.latitude, self.geo.longitude
        );

        let response = match get(&url, Some(&self.credentials.token().await.to_owned()), 100).await
        {
            Ok(r) => r,
            Err(_) => return Err(AppError::NotFound),
        };

        let value: HashMap<String, Value> = response.json().await.unwrap();

        let stream_url = value.get("streamUrl").unwrap().as_str().unwrap();
        let m3u_data = get(stream_url, None, 100)
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let master_playlist = hls_m3u8::MasterPlaylist::try_from(m3u_data.as_str());

        // If there's a master playlist, parse it and get the highest quality stream, else we already have the
        // correct URL.
        match master_playlist {
            Ok(mp) => Ok(Mutex::new(highest_quality_url(
                mp.variant_streams,
                stream_url,
            ))),
            Err(_) => Ok(Mutex::new(stream_url.to_owned())),
        }
    }

    /// Returns the `Geo` that is associated with this service
    fn geo(&self) -> Arc<Geo> {
        self.geo.clone()
    }

    /// Returns the UUID of this service
    fn uuid(&self) -> String {
        self.uuid.to_owned()
    }

    /// Returns the zipcode (if set) of this service
    fn zipcode(&self) -> String {
        if let Some(z) = &self.zipcode {
            z.to_owned()
        } else {
            "".to_string()
        }
    }

    /// Returns the services associated to this service. In the case of locast service implementation,
    /// this is an empty vector.
    fn services(&self) -> Vec<LocastServiceArc> {
        Vec::new()
    }
}

/// Sort the variant streams by bandwith (desc), pick the top one and return the full URL
fn highest_quality_url(
    mut variant_streams: Vec<hls_m3u8::tags::VariantStream>,
    stream_url: &str,
) -> String {
    variant_streams.sort_by(|a, b| {
        let ea = match a {
            hls_m3u8::tags::VariantStream::ExtXStreamInf { stream_data, .. } => {
                stream_data.bandwidth()
            }
            _ => 0,
        };
        let eb = match b {
            hls_m3u8::tags::VariantStream::ExtXStreamInf { stream_data, .. } => {
                stream_data.bandwidth()
            }
            _ => 0,
        };
        ea.cmp(&eb)
    });
    let variant = variant_streams.pop().unwrap();
    let variant_url = match variant {
        hls_m3u8::tags::VariantStream::ExtXStreamInf { uri, .. } => uri,
        _ => Cow::Borrowed(""),
    };
    Url::parse(&stream_url)
        .unwrap()
        .join(&variant_url.to_string())
        .unwrap()
        .to_string()
}

impl fmt::Display for LocastService {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "LocastService{{ zipcode: {:?}, uuid: {:?}, geo: {:?} }}",
            self.zipcode, self.uuid, self.geo
        )
    }
}

/// Start a `LocastService` updater thread
fn start_updater_thread(
    config: &Arc<Config>,
    stations: &Stations,
    geo: &Arc<Geo>,
    credentials: &Arc<LocastCredentials>,
    fcc_facilities: &Arc<FCCFacilities>,
) {
    // TODO: Can this be done nicer?
    let thread_stations = stations.clone();
    let thread_config = config.clone();
    let thread_geo = geo.clone();
    let thread_credentials = credentials.clone();
    let thread_facilities = fcc_facilities.clone();
    let thread_timeout = config.cache_timeout;

    task::spawn(async move {
        loop {
            sleep(Duration::from_secs(thread_timeout)).await;
            let ls = locast_stations(
                &thread_geo.DMA,
                thread_config.days,
                &thread_credentials.token().await,
            )
            .await;
            let new_stations =
                build_stations(ls, &thread_geo, &thread_config, &thread_facilities).await;
            let mut stations = thread_stations.lock().await;
            *stations = new_stations;
        }
    });
}

/// Retrieve and enrich station data
async fn build_stations(
    locast_stations: Vec<Station>,
    geo: &Geo,
    config: &Arc<Config>,
    fcc_facilities: &Arc<FCCFacilities>,
) -> Vec<Station> {
    info!(
        "Loading stations for {} (cache: {}, cache timeout: {}, days: {})..",
        geo.name, !config.disable_station_cache, config.cache_timeout, config.days
    );

    let mut stations: Vec<Station> = Vec::new();

    // Iterate over all locast stations for this service
    for mut station in locast_stations.into_iter() {
        // Add some data we need for display
        station.timezone = geo.timezone.to_owned();
        station.city = Some(geo.name.to_owned());

        // See if we can get the channel number from the call sign (i.e. X.Y NAME)
        let channel_from_call_sign = Regex::new(r"(\d+\.\d+) .+")
            .unwrap()
            .captures(&station.callSign)
            .map(|c| c.get(1).map_or("", |m| m.as_str()));

        // If the station's call sign is in the format "X.Y NAME", use X.Y as the channel number,
        // otherwise, we'll have to lookup the channel number using the name or the call sign.
        // And if we can't find the channel, we panic.
        let c = if let Some(channel) = channel_from_call_sign {
            Some(channel.to_string())
        } else if let Some((call_sign, sub_channel)) =
            detect_callsign(&station.name).or_else(|| detect_callsign(&station.callSign))
        {
            let dma = geo.DMA.parse::<i64>().unwrap();
            Some(fcc_facilities.lookup(dma, call_sign, sub_channel).await)
        } else {
            panic!(
                "Channel {}, call sign: {} not found!",
                &station.name, &station.callSign
            );
        };
        station.channel = c;
        // Rewrite the callsign to remove the channel number
        station.callSign = crate::utils::name_only(&station.callSign).to_string();
        stations.push(station);
    }
    stations
}

/// Get all stations from locast.org by specifying how many days in the future we would
/// like station information.
async fn locast_stations(dma: &str, days: u8, token: &str) -> Vec<Station> {
    let start_time = Utc::now().format("%Y-%m-%dT00:00:00-00:00").to_string();
    let uri = format!(
        "{}/{}?startTime={}&hours={}",
        STATIONS_URL,
        dma,
        start_time,
        days * 24
    );
    crate::utils::get(&uri, Some(token), 100)
        .await
        .unwrap()
        .json::<Vec<Station>>()
        .await
        .unwrap()
}

/// Detect a call sign from a string.
fn detect_callsign(input: &str) -> Option<(&str, &str)> {
    let re = Regex::new(r"^([KW][A-Z]{2,3})[A-Z]{0,2}(\d{0,2})$").unwrap();
    let caps = re.captures(input)?;
    let call_sign = caps.get(1).map_or("default", |m| m.as_str());
    let sub_channel = caps.get(2).map_or("default", |m| m.as_str());
    Some((call_sign, sub_channel))
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Geo {
    pub latitude: f64,
    pub longitude: f64,
    pub DMA: String,
    pub name: String,
    pub active: bool,
    pub timezone: Option<String>,
}
async fn geo_from(zipcode: &Option<String>) -> Geo {
    let uri = match zipcode {
        Some(z) => format!("{}/zip/{}", DMA_URL, z),
        None => String::from(IP_URL),
    };

    let mut geo = crate::utils::get(&uri, None, 100)
        .await
        .unwrap()
        .json::<Geo>()
        .await
        .unwrap();
    geo.timezone = Some(tz_search::lookup(geo.latitude, geo.longitude).unwrap());
    geo
}
