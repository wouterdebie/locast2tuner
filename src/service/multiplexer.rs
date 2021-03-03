use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use log::info;

use crate::{
    config::Config,
    service::{Geo, LocastServiceArc, Station, StationProvider, Stations},
};
pub struct Multiplexer {
    services: Vec<LocastServiceArc>,
    config: Arc<Config>,
    station_id_service_map: Mutex<HashMap<String, LocastServiceArc>>,
}

impl Multiplexer {
    pub fn new(services: Vec<LocastServiceArc>, config: Arc<Config>) -> MultiplexerArc {
        Arc::new(Multiplexer {
            services,
            config,
            station_id_service_map: Mutex::new(HashMap::new()),
        })
    }
}

type MultiplexerArc = Arc<Multiplexer>;

impl StationProvider for Arc<Multiplexer> {
    fn station_stream_uri(&self, id: &str) -> String {
        // Make sure the station_id_service_map is loaded. Feels wrong to do it like this though.. Needs refactoring.
        self.stations();
        self.station_id_service_map
            .lock()
            .unwrap()
            .get(&id.to_string())
            .unwrap()
            .station_stream_uri(id)
    }

    fn stations(&self) -> Stations {
        let mut all_stations: Vec<Station> = Vec::new();
        let services = self.services.clone();
        let services_len = services.len();
        for (i, service) in services.into_iter().enumerate() {
            let stations_mutex = service.stations();
            let stations = stations_mutex.lock().unwrap();
            for mut station in stations.iter().map(|s| s.clone()) {
                if self.config.remap {
                    let channel = station.channel.as_ref().unwrap();
                    if let Ok(c) = channel.parse::<usize>() {
                        station.channel_remapped = Some((c + 100 * i).to_string());
                    } else if let Ok(c) = channel.parse::<f32>() {
                        station.channel_remapped = Some((c + 100.0 * i as f32).to_string());
                    } else {
                        panic!(format!("Could not remap {}", channel));
                    };

                    // Convoluted.. let's fix this sometime..
                    let new_call_sign = station
                        .callSign
                        .replace(channel, &station.channel_remapped.as_ref().unwrap());
                    station.callSign_remapped = Some(new_call_sign);
                }
                self.station_id_service_map
                    .lock()
                    .unwrap()
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
            DMA: "000".to_string(),
            name: "Multiplexer".to_string(),
            active: true,
            timezone: None,
        })
    }

    fn uuid(&self) -> String {
        self.config.uuid.to_owned()
    }

    fn zipcode(&self) -> String {
        "".to_string()
    }

    fn services(&self) -> Vec<LocastServiceArc> {
        self.services.clone()
    }
}
