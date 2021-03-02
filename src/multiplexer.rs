use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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
        self.station_id_service_map
            .lock()
            .unwrap()
            .get(id)
            .unwrap()
            .station_stream_uri(id)
    }

    fn stations(&self) -> Stations {
        let mut all_stations: Vec<Station> = Vec::new();
        let services = self.services.clone();
        for service in services {
            let stations_mutex = service.stations();
            let stations = stations_mutex.lock().unwrap();
            for station in stations.iter().map(|s| s.clone()) {
                self.station_id_service_map
                    .lock()
                    .unwrap()
                    .insert(station.id.to_string(), service.clone());
                all_stations.push(station);
            }
        }
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
}
