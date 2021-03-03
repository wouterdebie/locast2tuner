use std::sync::Arc;

use super::{station::Stations, Geo, LocastServiceArc};

pub trait StationProvider {
    fn station_stream_uri(&self, id: &str) -> String;
    fn stations(&self) -> Stations;
    fn geo(&self) -> Arc<Geo>;
    fn uuid(&self) -> String;
    fn zipcode(&self) -> String;
    fn services(&self) -> Vec<LocastServiceArc>;
}
