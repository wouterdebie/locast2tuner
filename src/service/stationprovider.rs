use super::{station::Stations, Geo, LocastServiceArc};
use async_trait::async_trait;
use futures::lock::Mutex;
use std::sync::Arc;

#[async_trait]
pub trait StationProvider {
    async fn station_stream_uri(&self, id: String) -> Mutex<String>;
    async fn stations(&self) -> Stations;
    fn geo(&self) -> Arc<Geo>;
    fn uuid(&self) -> String;
    fn zipcode(&self) -> String;
    fn services(&self) -> Vec<LocastServiceArc>;
}
