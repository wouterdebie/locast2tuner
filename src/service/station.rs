use std::sync::Arc;

use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Station {
    pub active: bool,
    pub callSign: String,
    pub channel: Option<String>,
    pub city: Option<String>,
    pub dma: i64,
    pub id: i64,
    pub listings: Vec<Listing>,
    pub logo226Url: Option<String>,
    pub logoUrl: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<i64>,
    pub stationId: String,
    pub timezone: Option<String>,
    pub tivoId: Option<i64>,
    pub transcodeId: i64,
    pub channel_remapped: Option<String>,
    pub callSign_remapped: Option<String>,
    pub remapped: Option<bool>,
}
pub type Stations = Arc<Mutex<Vec<Station>>>;

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Listing {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub airdate: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audioProperties: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directors: Option<String>,
    pub duration: i64,
    pub entityType: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episodeNumber: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episodeTitle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genres: Option<String>,
    pub hasImageArtwork: bool,
    pub hasSeriesArtwork: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isNew: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferredImage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferredImageHeight: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferredImageWidth: Option<i16>,
    pub programId: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releaseDate: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releaseYear: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seasonNumber: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seriesId: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortDescription: Option<String>,
    pub showType: String,
    pub startTime: i64,
    pub stationId: i64,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topCast: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub videoProperties: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChannelRemapEntry {
    pub original_call_sign: String,
    pub remap_call_sign: String,
    pub original_channel: String,
    pub remap_channel: String,
    pub city: String,
    pub active: bool,
    pub remapped: bool,
}
