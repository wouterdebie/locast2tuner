use super::station::ChannelRemapEntry;
use crate::{
    config::Config,
    errors::AppError,
    service::{Geo, LocastService, Station, StationProvider, Stations},
};
use async_trait::async_trait;
use futures::lock::Mutex;
use log::info;
use std::{collections::HashMap, fs::File, sync::Arc};

/// Multiplex `LocastService` objects. `Multiplexer` implements the `StationProvider` trait
/// and can act as a LocastService.
pub struct Multiplexer {
    services: Vec<Arc<LocastService>>,
    config: Arc<Config>,
    station_id_service_map: Mutex<HashMap<String, Arc<LocastService>>>,
    channel_remap: Option<HashMap<String, ChannelRemapEntry>>,
}

impl Multiplexer {
    /// Create a new `Multiplexer` with a vector of `Arc<LocastService>s` and a `Config`
    pub fn new(services: Vec<Arc<LocastService>>, config: Arc<Config>) -> Arc<Multiplexer> {
        let channel_remap = match &config.remap_file {
            Some(f) => {
                let file = File::open(f).unwrap();
                let c: HashMap<String, ChannelRemapEntry> = serde_json::from_reader(file).unwrap();
                Some(c)
            }
            None => None,
        };
        Arc::new(Multiplexer {
            services,
            config,
            station_id_service_map: Mutex::new(HashMap::new()),
            channel_remap,
        })
    }
}

#[async_trait]
impl StationProvider for Arc<Multiplexer> {
    /// Get the stream URL for a locast station id.
    async fn station_stream_uri(&self, id: &str) -> Result<Mutex<String>, AppError> {
        // Make sure the station_id_service_map is loaded. Feels wrong to do it like this though.. Needs refactoring.
        self.stations().await;

        let service = match self.station_id_service_map.lock().await.get(&id.to_owned()) {
            Some(s) => s.clone(),
            None => return Err(AppError::NotFound),
        };

        service.station_stream_uri(id).await
    }

    /// Get all stations for all `LocastService`s.
    async fn stations(&self) -> Stations {
        let mut all_stations: Vec<Station> = Vec::new();
        let services = self.services.clone();
        let services_len = services.len();
        for (i, service) in services.into_iter().enumerate() {
            let stations_mutex = service.stations().await;

            let stations = stations_mutex.lock().await;
            for mut station in stations.iter().map(|s| s.clone()) {
                if self.config.remap {
                    let channel = station.channel.as_ref().unwrap();
                    if let Ok(c) = channel.parse::<usize>() {
                        station.channel_remapped = Some((c + 100 * i).to_string());
                    } else if let Ok(c) = channel.parse::<f32>() {
                        station.channel_remapped = Some((c + 100.0 * i as f32).to_string());
                    } else {
                        panic!("Could not remap {}", channel);
                    };

                    station.callSign_remapped = Some(station.callSign.clone());
                    station.remapped = Some(true)
                } else if self.channel_remap.is_some() {
                    // Look if the channel is is remapped in the channel map
                    let channel_remap = self.channel_remap.as_ref().unwrap();
                    let key = format!("channel.{}", station.id);
                    match channel_remap.get(&key) {
                        Some(r) if r.remapped => {
                            station.channel_remapped = Some(r.remap_channel.clone());
                            station.callSign_remapped = Some(r.remap_call_sign.clone());
                            station.active = r.active;
                            station.remapped = Some(r.remapped);
                            debug!(
                                "Remap -  {} {} => {} {}",
                                station.channel.clone().unwrap(),
                                station.callSign,
                                station.channel_remapped.clone().unwrap(),
                                station.callSign_remapped.clone().unwrap()
                            );
                        }
                        _ => {}
                    }
                }
                self.station_id_service_map
                    .lock()
                    .await
                    .insert(station.id.to_string(), service.clone());
                all_stations.push(station);
            }
        }
        info!(
            "Got {} stations for {} cities",
            all_stations.len(),
            services_len
        );
        Arc::new(Mutex::new(all_stations))
    }

    fn geo(&self) -> Arc<crate::service::Geo> {
        Arc::new(Geo {
            latitude: 0.0,
            longitude: 0.0,
            DMA: "000".to_owned(),
            name: "Multiplexer".to_owned(),
            active: true,
            timezone: None,
        })
    }

    fn uuid(&self) -> String {
        self.config.uuid.to_owned()
    }

    fn zipcodes(&self) -> Vec<String> {
        vec![]
    }

    fn services(&self) -> Vec<Arc<LocastService>> {
        self.services.clone()
    }
}
