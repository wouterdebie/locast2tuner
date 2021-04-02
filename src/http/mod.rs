mod streaming;
mod xml_templates;
use self::streaming::StreamBody;
use crate::{config::Config, service::stationprovider::StationProvider, utils::Or};
use actix_web::middleware::Compat;
use actix_web::middleware::Condition;
use actix_web::middleware::Logger;
use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use futures::future;
use log::info;
use prettytable::{cell, format, row, Table};
use reqwest::header::LOCATION;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use string_builder::Builder;

const NETWORKS: [&'static str; 6] = ["ABC", "CBS", "NBC", "FOX", "CW", "PBS"];

struct AppState<T: StationProvider> {
    config: Arc<Config>,
    service: T,
    port: u16,
    station_scan: Mutex<bool>,
}

#[actix_web::main]
pub async fn start<T: 'static + StationProvider + Sync + Send + Clone>(
    services: Vec<T>,
    config: Arc<Config>,
) -> std::io::Result<()> {
    let reporting_services = services.clone();
    let servers: Vec<Server> = services
        .into_iter()
        .enumerate()
        .map(|(i, service)| {
            let port = config.port + i as u16;
            let bind_address = &config.bind_address;
            info!(
                "Starting http server for {} on http://{}:{}",
                service.geo().name,
                bind_address,
                port
            );
            let app_state = web::Data::new(AppState::<T> {
                config: config.clone(),
                service: service.clone(),
                port,
                station_scan: Mutex::new(false),
            });

            let verbose = config.verbose;

            HttpServer::new(move || {
                App::new()
                    .wrap(Condition::new(verbose > 0, Compat::new(Logger::default())))
                    .app_data(app_state.clone())
                    .route("/", web::get().to(device_xml::<T>))
                    .route("/device.xml", web::get().to(device_xml::<T>))
                    .route("/lineup.xml", web::get().to(lineup_xml::<T>))
                    .route("/discover.json", web::get().to(discover::<T>))
                    .route("/lineup_status.json", web::get().to(lineup_status::<T>))
                    .route("/tuner.m3u", web::get().to(tuner_m3u::<T>))
                    .route("/lineup.json", web::get().to(lineup_json::<T>))
                    .route("/epg", web::get().to(epg::<T>))
                    .route("/epg.xml", web::get().to(epg_xml::<T>))
                    .route("/config", web::get().to(show_config::<T>))
                    .route("/lineup.post", web::post().to(lineup_post))
                    .service(web::resource("/watch/{id}.m3u").route(web::get().to(watch_m3u::<T>)))
                    .service(web::resource("/watch/{id}").route(web::get().to(watch::<T>)))
            })
            .bind((bind_address.to_owned(), port))
            .unwrap()
            .run()
        })
        .collect();

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
async fn device_xml<T: StationProvider>(data: web::Data<AppState<T>>) -> HttpResponse {
    let result = xml_templates::device_xml::<T>(&data.config, &data.service, data.port);
    HttpResponse::Ok().content_type("text/xml").body(result)
}

async fn lineup_xml<T: StationProvider>(data: web::Data<AppState<T>>) -> HttpResponse {
    let stations_mutex = data.service.stations();
    let stations = &stations_mutex.lock().unwrap();
    let result = xml_templates::lineup_xml(&data.config, stations, data.port);
    HttpResponse::Ok().content_type("text/xml").body(result)
}

async fn epg_xml<T: StationProvider>(data: web::Data<AppState<T>>) -> impl Responder {
    let stations_mutex = data.service.stations();
    let stations = &stations_mutex.lock().unwrap();
    let result = xml_templates::epg_xml(stations);
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

async fn discover<T: StationProvider>(data: web::Data<AppState<T>>) -> impl Responder {
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
        BaseURL: format!("http://{}:{}", data.config.bind_address, data.port),
        LineupURL: format!(
            "http://{}:{}/lineup.json",
            data.config.bind_address, data.port
        ),
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
    let station_scan = data.station_scan.lock().unwrap();
    let response = if *station_scan {
        LineupStatus {
            ScanInProgress: true,
            Progress: 50,
            Found: 6,
            SourceList: None,
        }
    } else {
        LineupStatus {
            ScanInProgress: true,
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
async fn tuner_m3u<T: StationProvider>(data: web::Data<AppState<T>>) -> impl Responder {
    let mut builder = Builder::default();
    builder.append("#EXTM3U\n");
    let stations_mutex = data.service.stations();
    let stations = stations_mutex.lock().unwrap();

    for station in stations.iter() {
        let call_sign_or_name = &station.callSign.or(&station.name).to_string();
        let call_sign = crate::utils::name_only(
            &station
                .callSign_remapped
                .as_ref()
                .unwrap_or(call_sign_or_name),
        );
        let city = station.city.as_ref().unwrap();
        let logo = station.logoUrl.or(&station.logo226Url);
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

        let url = format!(
            "http://{}:{}/watch/{}.m3u",
            &data.config.bind_address, &data.port, &station.id
        );
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

async fn lineup_json<T: StationProvider>(data: web::Data<AppState<T>>) -> impl Responder {
    let stations_mutex = data.service.stations();
    let stations = stations_mutex.lock().unwrap();

    let lineup: Vec<LineupJson> = stations
        .iter()
        .map(|station| {
            let url = format!(
                "http://{}:{}/watch/{}",
                &data.config.bind_address, &data.port, &station.id
            );
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
async fn show_config<T: StationProvider>(data: web::Data<AppState<T>>) -> impl Responder {
    let mut config = (*data.config).clone();
    config.password = "*******".to_string();
    HttpResponse::Ok().json(&config)
}

async fn epg<T: StationProvider>(data: web::Data<AppState<T>>) -> impl Responder {
    let stations_mutex = data.service.stations();
    let stations = &*stations_mutex.lock().unwrap();
    HttpResponse::Ok().json(stations)
}

async fn watch_m3u<T: 'static + StationProvider>(req: HttpRequest) -> impl Responder {
    let id = req.match_info().get("id").unwrap().to_string();
    let service = &req.app_data::<web::Data<AppState<T>>>().unwrap().service;
    let url = service.station_stream_uri(id).await;
    HttpResponse::TemporaryRedirect()
        .append_header((LOCATION, url))
        .finish()
}

async fn watch<T: 'static + StationProvider>(req: HttpRequest) -> impl Responder {
    let id = req.match_info().get("id").unwrap().to_string();
    let service = &req.app_data::<web::Data<AppState<T>>>().unwrap().service;
    let url = service.station_stream_uri(id).await;

    let stream = StreamBody::new(url);

    HttpResponse::Ok()
        .content_type("video/mpeg; codecs='avc1.4D401E'")
        .streaming(stream)
}

async fn lineup_post(_req: HttpRequest) -> impl Responder {
    HttpResponse::NoContent()
}
