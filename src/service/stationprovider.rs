use std::{pin::Pin, sync::Arc};

use futures::Future;

use super::{station::Stations, Geo, LocastServiceArc};

pub trait StationProvider {
    fn station_stream_uri(&self, id: String) -> Pin<Box<dyn Future<Output = String> + '_>>;
    fn stations(&self) -> Stations;
    fn geo(&self) -> Arc<Geo>;
    fn uuid(&self) -> String;
    fn zipcode(&self) -> String;
    fn services(&self) -> Vec<LocastServiceArc>;
}
