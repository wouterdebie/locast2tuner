mod templates;
use crate::utils::base_url;
use crate::{config::Config, service::stationprovider::StationProvider, utils::Or};
use actix_web::middleware::Condition;
use actix_web::middleware::Logger;
use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::{middleware::Compat, Error};
use chrono::{DateTime, Utc};
use futures::{future, lock::Mutex, stream, Stream};
use log::info;
use prettytable::{cell, format, row, Table};
use reqwest::{header::LOCATION, Url};
use serde::Serialize;
use std::convert::TryFrom;
use std::{collections::VecDeque, sync::Arc};
use string_builder::Builder;
use uuid::Uuid;

const NETWORKS: [&'static str; 6] = ["ABC", "CBS", "NBC", "FOX", "CW", "PBS"];

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
                service: service.clone(),
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

    for station in stations.lock().await.iter() {
        let call_sign_or_name = &station.callSign.or(&station.name).to_string();
        let call_sign = crate::utils::name_only(
            &station
                .callSign_remapped
                .as_ref()
                .unwrap_or(call_sign_or_name),
        );
        let city = station.city.as_ref().unwrap();
        let logo = &station
            .logoUrl
            .as_ref()
            .or(station.logo226Url.as_ref())
            .unwrap();
        let channel = &station
            .channel_remapped
            .as_ref()
            .unwrap_or(station.channel.as_ref().unwrap());
        let groups = if NETWORKS.contains(&call_sign) {
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
async fn show_config<T: 'static + StationProvider>(req: HttpRequest) -> impl Responder {
    let mut config = (*req.app_data::<web::Data<AppState<T>>>().unwrap().config).clone();

    if req.query_string() != "show_password" {
        config.password = "*******".to_string();
    }

    let result = toml::to_string(&config).unwrap();
    HttpResponse::Ok().content_type("text/plain").body(result)
}

async fn epg<T: StationProvider>(data: web::Data<AppState<T>>) -> impl Responder {
    let stations_mutex = data.service.stations();
    let stations = &*stations_mutex.await;
    HttpResponse::Ok().json(stations.lock().await)
}

async fn watch_m3u<T: 'static + StationProvider>(req: HttpRequest) -> impl Responder {
    let id = req.match_info().get("id").unwrap().to_string();
    let service = &req.app_data::<web::Data<AppState<T>>>().unwrap().service;
    let url_mutex = service.station_stream_uri(id).await;
    let url = url_mutex.lock().await;

    HttpResponse::TemporaryRedirect()
        .append_header((LOCATION, &*url.as_str()))
        .finish()
}

async fn watch<T: 'static + StationProvider>(req: HttpRequest) -> impl Responder {
    let id = req.match_info().get("id").unwrap().to_string();
    let service = &req.app_data::<web::Data<AppState<T>>>().unwrap().service;
    let url_mutex = service.station_stream_uri(id).await;
    let url = url_mutex.lock().await;
    let stream = get_stream(&*url);

    HttpResponse::Ok()
        .content_type("video/mpeg; codecs='avc1.4D401E'")
        .streaming(Box::pin(stream))
}

struct StreamState {
    segments: VecDeque<Segment>,
    url: String,
    stream_id: String,
    start_time: DateTime<Utc>,
    seconds_served: f32,
}

fn get_stream(url: &str) -> impl Stream<Item = Result<bytes::Bytes, Error>> {
    // Build helper struct
    let state = StreamState {
        segments: VecDeque::new(),
        url: url.to_owned(),
        stream_id: Uuid::new_v4().to_string()[0..7].to_string(),
        start_time: Utc::now(),
        seconds_served: 0.0,
    };

    stream::unfold(state, |mut state| async move {
        let m3u_data = crate::utils::get(&state.url, None)
            .await
            .text()
            .await
            .unwrap();
        let media_playlist = hls_m3u8::MediaPlaylist::try_from(m3u_data.as_str()).unwrap();
        let base_url = base_url(Url::parse(&state.url).unwrap());

        for media_segment in media_playlist.segments {
            let (_i, ms) = media_segment;
            let absolute_uri = base_url.join(ms.uri()).unwrap();
            let s = Segment {
                url: absolute_uri.to_string(),
                played: false,
                duration: ms.duration.duration(),
            };
            if !state.segments.contains(&s) {
                info!("Stream {} - added segment {:?}", state.stream_id, &s.url);
                state.segments.push_back(s);
            }
        }

        if state.segments.len() >= 30 {
            info!("Stream {} - draining 10 segments", state.stream_id);
            state.segments.drain(0..10);
        }

        // Find first unplayed segment
        let first = state.segments.iter_mut().find(|s| !s.played).unwrap();

        let runtime = Utc::now() - state.start_time;
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

        let chunk = crate::utils::get(&first.url, None)
            .await
            .bytes()
            .await
            .unwrap()
            .to_vec();
        first.played = true;
        info!(
            "Stream {} - playing: segment {:?}",
            state.stream_id, first.url
        );

        state.seconds_served += first.duration.as_secs_f32();
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
