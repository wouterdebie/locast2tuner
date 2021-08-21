use crate::errors::AppError;

use super::{station::Stations, Geo, LocastService};
use async_trait::async_trait;
use futures::lock::Mutex;
use std::sync::Arc;

#[async_trait]
pub trait StationProvider {
    async fn station_stream_uri(&self, id: &str) -> Result<Mutex<String>, AppError>;
    async fn stations(&self) -> Stations;
    fn geo(&self) -> Arc<Geo>;
    fn uuid(&self) -> String;
    fn zipcodes(&self) -> Vec<String>;
    fn services(&self) -> Vec<Arc<LocastService>>;
}
