mod templates;
use crate::{
    config::Config,
    service::{station::ChannelRemapEntry, stationprovider::StationProvider},
    utils::Or,
};
use actix_web::middleware::Logger;
use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::{middleware::Compat, Error};
use actix_web::{middleware::Condition, ResponseError};
use chrono::{DateTime, Utc};
use futures::{future, lock::Mutex, stream, Stream};
use log::info;
use prettytable::{cell, format, row, Table};
use reqwest::{header::LOCATION, Url};
use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::{collections::VecDeque, sync::Arc};
use string_builder::Builder;
use uuid::Uuid;

const NETWORKS: [&str; 6] = ["ABC", "CBS", "NBC", "FOX", "CW", "PBS"];

/// Struct that is passed to HTTP handlers that contains config, the service that can be used to
/// lookup locast data, etc.
struct AppState<T: StationProvider> {
    config: Arc<Config>,
    service: T,
    station_scan: Mutex<bool>,
}

/// Start the HTTP server that will handle media server requests
pub async fn start<T: 'static + StationProvider + Sync + Send + Clone>(
    services: Vec<T>,
    config: Arc<Config>,
) -> std::io::Result<()> {
    let reporting_services = services.clone();
    // Start a server for each service that is passed in
    let servers: Vec<Server> = services
        .into_iter()
        .enumerate()
        .map(|(i, service)| {
            // Create port and address
            let port = config.port + i as u16;
            let bind_address = &config.bind_address;
            info!(
                "Starting http server for {} on http://{}:{}",
                service.geo().name,
                bind_address,
                port
            );

            // Construct some app_state we can pass around
            let app_state = web::Data::new(AppState::<T> {
                config: config.clone(),
                service,
                station_scan: Mutex::new(false),
            });

            let verbose = config.verbose;

            HttpServer::new(move || {
                App::new()
                    // Log HTTP requests if verbosity > 0
                    .wrap(Condition::new(verbose > 0, Compat::new(Logger::default())))
                    .app_data(app_state.clone())
                    .route("/", web::get().to(device_xml::<T>))
                    .route("/config", web::get().to(show_config::<T>))
                    .route("/device.xml", web::get().to(device_xml::<T>))
                    .route("/discover.json", web::get().to(discover::<T>))
                    .route("/epg.xml", web::get().to(epg_xml::<T>))
                    .route("/epg", web::get().to(epg::<T>))
                    .route("/lineup_status.json", web::get().to(lineup_status::<T>))
                    .route("/lineup.json", web::get().to(lineup_json::<T>))
                    .route("/lineup.post", web::post().to(lineup_post))
                    .route("/lineup.xml", web::get().to(lineup_xml::<T>))
                    .route("/map.json", web::get().to(map_json::<T>))
                    .route("/tuner.m3u", web::get().to(tuner_m3u::<T>))
                    .service(web::resource("/watch/{id}.m3u").route(web::get().to(watch_m3u::<T>)))
                    .service(web::resource("/watch/{id}").route(web::get().to(watch::<T>)))
            })
            .bind((bind_address.to_owned(), port))
            .unwrap()
            .run()
        })
        .collect();

    // Report on what has been started
    if config.multiplex {
        info!("Tuners:");
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        table.set_titles(row!["City", "Zip code", "DMA", "UUID", "Timezone"]);
        for s in reporting_services[0].services() {
            table.add_row(row![
                s.geo().name,
                s.zipcode(),
                s.geo().DMA,
                s.uuid(),
                s.geo().timezone.as_ref().unwrap_or(&"".to_string())
            ]);
        }

        for line in table.to_string().lines() {
            info!(" {}", line);
        }
        info!("");
        info!("Multiplexer:");
        let url = format!("http://{}:{}", config.bind_address, config.port);
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        table.set_titles(row!["UID", "URL"]);
        table.add_row(row![reporting_services[0].uuid(), url]);
        for line in table.to_string().lines() {
            info!(" {}", line);
        }
    } else {
        info!("Tuners:");
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        table.set_titles(row!["City", "Zip code", "DMA", "UUID", "Timezone", "URL"]);
        for is in reporting_services.iter().enumerate() {
            let (i, s) = is;
            let port = config.port + i as u16;
            let url = format!("http://{}:{}", config.bind_address, port);
            table.add_row(row![
                s.geo().name,
                s.zipcode(),
                s.geo().DMA,
                s.uuid(),
                s.geo().timezone.as_ref().unwrap_or(&"".to_string()),
                url
            ]);
        }
        for line in table.to_string().lines() {
            info!(" {}", line);
        }
    }

    info!("locast2tuner started..");
    future::try_join_all(servers).await?;
    Ok(())
}

async fn device_xml<T: 'static + StationProvider>(req: HttpRequest) -> HttpResponse {
    let data = &req.app_data::<web::Data<AppState<T>>>().unwrap();
    let host = req.connection_info().host().to_string();
    let result = templates::device_xml::<T>(&data.config, &data.service, host);
    HttpResponse::Ok().content_type("text/xml").body(result)
}

async fn lineup_xml<T: 'static + StationProvider>(req: HttpRequest) -> HttpResponse {
    let data = &req.app_data::<web::Data<AppState<T>>>().unwrap();
    let host = req.connection_info().host().to_string();
    let stations_mutex = data.service.stations();
    let stations = stations_mutex.await;
    let result = templates::lineup_xml(&*stations.lock().await, host);
    HttpResponse::Ok().content_type("text/xml").body(result)
}

async fn epg_xml<T: StationProvider>(data: web::Data<AppState<T>>) -> impl Responder {
    let stations_mutex = data.service.stations();
    let stations = stations_mutex.await;
    let result = templates::epg_xml(&*stations.lock().await);
    HttpResponse::Ok().content_type("text/xml").body(result)
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct DiscoverData {
    FriendlyName: String,
    Manufacturer: String,
    ModelNumber: String,
    FirmwareName: String,
    TunerCount: u8,
    FirmwareVersion: String,
    DeviceID: String,
    DeviceAuth: String,
    BaseURL: String,
    LineupURL: String,
}

async fn discover<T: 'static + StationProvider>(req: HttpRequest) -> HttpResponse {
    let data = &req.app_data::<web::Data<AppState<T>>>().unwrap();
    let host = req.connection_info().host().to_string();
    let uuid = &data.config.uuid;
    let device_id = usize::from_str_radix(&uuid[..8], 16).unwrap();
    let checksum = crate::utils::hdhr_checksum(device_id); // TODO: FIX!
    let valid_id = format!("{:x}", checksum + device_id);
    let response = DiscoverData {
        FriendlyName: data.service.geo().name.clone(),
        Manufacturer: "locast2dvr".to_string(),
        ModelNumber: data.config.device_model.clone(),
        FirmwareName: data.config.device_firmware.clone(),
        TunerCount: data.config.tuner_count,
        FirmwareVersion: data.config.device_version.clone(),
        DeviceID: valid_id,
        DeviceAuth: "locast2dvr".to_string(),
        BaseURL: format!("http://{}", host),
        LineupURL: format!("http://{}/lineup.json", host),
    };

    HttpResponse::Ok().json(&response)
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct LineupStatus {
    ScanInProgress: bool,
    Progress: u8,
    Found: u8,
    SourceList: Option<Vec<String>>,
}
async fn lineup_status<T: StationProvider>(data: web::Data<AppState<T>>) -> impl Responder {
    let station_scan = data.station_scan.lock().await;
    let response = if *station_scan {
        LineupStatus {
            ScanInProgress: true,
            Progress: 50,
            Found: 6,
            SourceList: None,
        }
    } else {
        LineupStatus {
            ScanInProgress: false,
            Progress: 50,
            Found: 6,
            SourceList: Some(vec!["Antenna".to_string()]),
        }
    };
    HttpResponse::Ok().json(&response)
}

impl Or for String {
    fn or<'a>(&'a self, other: &'a str) -> &str {
        if !self.is_empty() {
            self
        } else {
            other
        }
    }
}
async fn tuner_m3u<T: 'static + StationProvider>(req: HttpRequest) -> HttpResponse {
    let data = &req.app_data::<web::Data<AppState<T>>>().unwrap();
    let host = req.connection_info().host().to_string();
    let mut builder = Builder::default();
    builder.append("#EXTM3U\n");
    let stations_mutex = data.service.stations();
    let stations = stations_mutex.await;

    for station in stations.lock().await.iter().filter(|s| s.active) {
        let call_sign_or_name = &station.callSign.or(&station.name).to_string();
        let call_sign = station
            .callSign_remapped
            .as_ref()
            .unwrap_or(call_sign_or_name);
        let city = station.city.as_ref().unwrap();
        let logo = &station
            .logoUrl
            .as_ref()
            .or_else(|| station.logo226Url.as_ref())
            .unwrap();
        let channel = &station
            .channel_remapped
            .as_ref()
            .unwrap_or_else(|| station.channel.as_ref().unwrap());
        let groups = if NETWORKS.contains(&call_sign.as_str()) {
            format!("{};Network", &city,)
        } else {
            city.to_owned()
        };

        let tvg_name = if data.config.multiplex {
            format!("{} ({})", call_sign, city)
        } else {
            call_sign.to_string()
        };

        builder.append(format!(
            "#EXTINF:-1 tvg-id=\"channel.{}\" tvg-name=\"{}\" tvg-logo=\"{}\" tvg-chno=\"{}\" group-title=\"{}\", {}",
            &station.id, &call_sign, &logo, &channel, &groups, &tvg_name
        ));

        let url = format!("http://{}/watch/{}.m3u", &host, &station.id);
        builder.append(format!("\n{}\n\n", url));
    }

    HttpResponse::Ok().body(builder.string().unwrap())
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct LineupJson {
    GuideNumber: String,
    GuideName: String,
    URL: String,
}

async fn lineup_json<T: 'static + StationProvider>(req: HttpRequest) -> HttpResponse {
    let data = &req.app_data::<web::Data<AppState<T>>>().unwrap();
    let host = req.connection_info().host().to_string();
    let stations_mutex = data.service.stations();
    let stations = stations_mutex.await;

    let lineup: Vec<LineupJson> = stations
        .lock()
        .await
        .iter()
        .filter(|s| s.active)
        .map(|station| {
            let url = format!("http://{}/watch/{}", &host, &station.id);
            LineupJson {
                GuideNumber: station
                    .channel_remapped
                    .as_ref()
                    .unwrap_or(&station.channel.as_ref().unwrap().to_owned())
                    .to_string(),
                GuideName: station.name.to_owned(),
                URL: url,
            }
        })
        .collect();

    HttpResponse::Ok().json(lineup)
}

async fn map_json<T: 'static + StationProvider>(req: HttpRequest) -> HttpResponse {
    let data = &req.app_data::<web::Data<AppState<T>>>().unwrap();
    let stations_mutex = data.service.stations();
    let stations = stations_mutex.await;

    let lineup: HashMap<String, ChannelRemapEntry> = stations
        .lock()
        .await
        .iter()
        .map(|station| {
            (
                format!("channel.{}", station.id),
                ChannelRemapEntry {
                    original_call_sign: station.callSign.clone(),
                    remap_call_sign: station
                        .callSign_remapped
                        .clone()
                        .or_else(|| Some(station.callSign.clone()))
                        .unwrap(),
                    original_channel: station.channel.clone().unwrap(),
                    remap_channel: station
                        .channel_remapped
                        .clone()
                        .or_else(|| Some(station.channel.clone().unwrap()))
                        .unwrap(),
                    city: station.city.clone().unwrap(),
                    active: station.active,
                    remapped: station.remapped.or(Some(false)).unwrap(),
                },
            )
        })
        .collect();

    let j = serde_json::to_string(&lineup).unwrap();
    HttpResponse::Ok().content_type("text/json").body(j)
}
async fn show_config<T: 'static + StationProvider>(req: HttpRequest) -> impl Responder {
    let mut config = (*req.app_data::<web::Data<AppState<T>>>().unwrap().config).clone();

    if req.query_string() != "show_password" {
        config.password = "*******".to_string();
    }

    let result = toml::to_string(&config).unwrap();
    HttpResponse::Ok().content_type("text/plain").body(result)
}

/// EPG in json format. This is pretty much the whole Vec<Station> we have built in memory.
/// Note that no additional filter is applied.
async fn epg<T: StationProvider>(data: web::Data<AppState<T>>) -> impl Responder {
    let stations_mutex = data.service.stations();
    let stations = &*stations_mutex.await;
    HttpResponse::Ok().json(stations.lock().await)
}

async fn watch_m3u<T: 'static + StationProvider>(req: HttpRequest) -> impl Responder {
    let id = req.match_info().get("id").unwrap();
    let service = &req.app_data::<web::Data<AppState<T>>>().unwrap().service;
    match service.station_stream_uri(id).await {
        Ok(url_mutex) => {
            let url = url_mutex.lock().await;

            HttpResponse::TemporaryRedirect()
                .append_header((LOCATION, &*url.as_str()))
                .finish()
        }
        Err(e) => e.error_response(),
    }
}

async fn watch<T: 'static + StationProvider>(req: HttpRequest) -> impl Responder {
    let id = req.match_info().get("id").unwrap();
    let service = &req.app_data::<web::Data<AppState<T>>>().unwrap().service;
    match service.station_stream_uri(id).await {
        Ok(url_mutex) => {
            let url = url_mutex.lock().await;
            let stream = get_stream::<T>(&*url, req);

            HttpResponse::Ok()
                .content_type("video/mpeg; codecs='avc1.4D401E'")
                .streaming(Box::pin(stream))
        }
        Err(e) => e.error_response(),
    }
}

struct StreamState {
    segments: VecDeque<Segment>,
    url: String,
    stream_id: String,
    start_time: DateTime<Utc>,
    seconds_served: f32,
    req: HttpRequest,
    count_down: f32,
}

static COUNT_DOWN: f32 = 9900.0; // 2:45h
fn get_stream<T: 'static + StationProvider>(
    url: &str,
    req: HttpRequest,
) -> impl Stream<Item = Result<bytes::Bytes, Error>> {
    // Build helper struct
    let state = StreamState {
        segments: VecDeque::new(),
        url: url.to_owned(),
        stream_id: Uuid::new_v4().to_string()[0..7].to_string(),
        start_time: Utc::now(),
        seconds_served: 0.0,
        count_down: COUNT_DOWN,
        req,
    };

    stream::unfold(state, |mut state| async move {
        // Refresh initial URL if we've been streaming for `COUNTDOWN seconds`
        if state.count_down < 0.0 {
            debug!("Stream {} -  URL expired: {}", state.stream_id, state.url);

            // Get the service and stream id from the state
            let id = state.req.match_info().get("id").unwrap();
            let service = &state
                .req
                .app_data::<web::Data<AppState<T>>>()
                .unwrap()
                .service;

            // Grab a new URL for this stream. If this fails, we end the stream.
            match service.station_stream_uri(id).await {
                Ok(url_mutex) => {
                    let url = url_mutex.lock().await;
                    debug!("Stream {} - New URL: {}", state.stream_id, &*url);
                    state.url = (&*url).to_owned();
                    state.count_down = COUNT_DOWN;
                }
                Err(_) => return None,
            }
        }

        // Try fetching the latest m3u playlist. Intermittent errors may happen here,
        // and when they do, we just skip updating until the next iteration.
        match crate::utils::get(&state.url, None, 5).await {
            Ok(r) => Some(r.text().await.unwrap()),
            Err(e) => {
                warn!("Unable to get m3u data, skipping a fetch.. {}", e);
                None
            }
        }
        .and_then(|m3u_data| {
            // Decode the m3u into a MediaPlaylist.
            match hls_m3u8::MediaPlaylist::from_str(m3u_data.as_str()) {
                Ok(p) => Some(p),
                Err(e) => {
                    warn!("Unable to decode media playlist, skipping a fetch.. {}", e);
                    None
                }
            }
        })
        .map(|media_playlist| {
            // Loop through each segment and add it to the segments vector
            // if it's not already in there.
            for media_segment in media_playlist.segments {
                let (_i, ms) = media_segment;
                let absolute_uri = Url::parse(&state.url)
                    .unwrap()
                    .join(ms.uri())
                    .unwrap()
                    .to_string();

                let s = Segment {
                    url: absolute_uri,
                    played: false,
                    duration: ms.duration.duration(),
                };

                if !state.segments.contains(&s) {
                    info!("Stream {} - added segment {:?}", state.stream_id, &s.url);
                    state.segments.push_back(s);
                }
            }
            Some(())
        });

        // In order to not grow the state, we drain 10 segments if we have more than
        // 30. I guess we should check for unplayed segments, but
        // if we have 30 segments with the first 10 unplayed, we're probably
        // screwed anyway.
        if state.segments.len() >= 30 {
            info!("Stream {} - draining 10 segments", state.stream_id);
            state.segments.drain(0..10);
        }

        // Find first unplayed segment and if there are no unplayed segments, we bail
        let first = match state.segments.iter_mut().find(|s| !s.played) {
            Some(s) => s,
            None => {
                warn!("No first segment found. Stopping stream..");
                return None;
            }
        };

        // Figure out how long we'll wait with serving the next segment. We do this
        // because otherwise, we constantly loop through our segment list and
        // make unnecessary calls to locast. This happens because serving a segment
        // instantly returns, rather than waiting for the client to play
        // the segment.
        let runtime = Utc::now() - state.start_time;

        // We want to be 50% of a segment behind. This is a nice trade-off
        // between having enough time to fetch the next segment and calling
        // locast too often.
        let target_diff = 0.5 * first.duration.as_secs_f32();

        let wait = if state.seconds_served > 0.0 {
            state.seconds_served - target_diff - (runtime.num_milliseconds() as f32 / 1000.0)
        } else {
            0.0
        };

        info!(
            "Serving {} ({} s) in {}s",
            &first.url,
            first.duration.as_secs_f32(),
            wait
        );

        if wait > 0.0 {
            tokio::time::sleep(tokio::time::Duration::from_secs_f32(wait)).await;
        }

        // Download the actual chunk from locast
        let chunk = match crate::utils::get(&first.url, None, 10).await {
            Err(e) => {
                warn!("No bytes fetched.. Stopping stream.. {}", e);
                return None;
            }
            Ok(r) => match r.bytes().await {
                Ok(b) => b.to_vec(),
                Err(e) => {
                    warn!("No bytes fetched.. Stopping stream.. {}", e);
                    return None;
                }
            },
        };

        // Mark the segment as played
        first.played = true;
        info!(
            "Stream {} - playing: segment {:?}",
            state.stream_id, first.url
        );

        state.seconds_served += first.duration.as_secs_f32();
        state.count_down -= first.duration.as_secs_f32();
        Some((Ok(actix_web::web::Bytes::from(chunk)), state))
    })
}

async fn lineup_post(_req: HttpRequest) -> impl Responder {
    HttpResponse::NoContent()
}

#[derive(Debug)]
struct Segment {
    url: String,
    played: bool,
    duration: std::time::Duration,
}
impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}
