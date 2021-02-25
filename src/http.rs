use crate::{config::Config, service::LocastService};
use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use askama::Template;
use futures::future;
use regex::Regex;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use string_builder::Builder;

const NETWORKS: [&'static str; 6] = ["ABC", "CBS", "NBC", "FOX", "CW", "PBS"];

#[derive(Template)] // this will generate the code...
#[template(path = "device.xml")] // using the template in this path, relative
                                 // to the `templates` dir in the crate root
struct DeviceXMLTemplate<'a> {
    // the name of the struct can be anything
    friendly_name: &'a str,
    device_model: &'a str,
    device_version: &'a str,
    uid: &'a str,
    host: &'a str,
    port: u16,
}

async fn device_xml(data: web::Data<AppState>) -> HttpResponse {
    let service = &data.service;
    let t = DeviceXMLTemplate {
        friendly_name: &service.geo.name,
        device_model: &data.config.device_model,
        device_version: &data.config.device_version,
        uid: &service.uuid,
        host: &data.config.bind_address,
        port: data.port,
    };
    HttpResponse::Ok()
        .content_type("text/xml")
        .body(t.render().unwrap())
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

async fn discover(data: web::Data<AppState>) -> impl Responder {
    let uuid = &data.config.uuid;
    let device_id = usize::from_str_radix(&uuid[..8], 16).unwrap();
    let checksum = crate::utils::hdhr_checksum(device_id); // TODO: FIX!
    let valid_id = format!("{:x}", checksum + device_id);
    let response = DiscoverData {
        FriendlyName: data.service.geo.name.clone(),
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

    HttpResponse::Ok().json(response)
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct LineupStatus {
    ScanInProgress: bool,
    Progress: u8,
    Found: u8,
    SourceList: Option<Vec<String>>,
}
async fn lineup_status(data: web::Data<AppState>) -> impl Responder {
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
    HttpResponse::Ok().json(response)
}

fn name_only(value: &str) -> &str {
    match Regex::new(r"\d+\.\d+ (.+)").unwrap().captures(value) {
        Some(c) => c.get(1).map_or("", |m| m.as_str()),
        None => &value,
    }
}

async fn tuner_m3u(data: web::Data<AppState>) -> impl Responder {
    let mut builder = Builder::default();
    builder.append("#EXTM3U\n");
    let stations_mutex = data.service.stations();
    let stations = stations_mutex.lock().unwrap();

    for station in stations.iter() {
        let call_sign = name_only(&station.callSign.or(&station.name));
        let city = &data.service.geo.name;
        let logo = station.logoUrl.or(&station.logo226Url);
        let channel = station.channel.as_ref().unwrap();
        let groups = if NETWORKS.contains(&call_sign) {
            format!("{};Network", &city,)
        } else {
            city.to_owned()
        };

        builder.append(format!(
            "#EXTINF:-1 tvg-id=\"channel.{}\" tvg-name=\"{}\" tvg-logo=\"{}\" tvg-chno=\"{}\" group-title=\"{}\", {}",
            &station.id, &call_sign, &logo, &channel, &groups, &call_sign
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

async fn lineup_json(data: web::Data<AppState>) -> impl Responder {
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
                GuideNumber: station.channel.as_ref().unwrap().to_owned(),
                GuideName: station.name.to_owned(),
                URL: url,
            }
        })
        .collect();

    HttpResponse::Ok().json(lineup)
}
async fn show_config(data: web::Data<AppState>) -> impl Responder {
    let mut config = (*data.config).clone();
    config.password = "*******".to_string();
    HttpResponse::Ok().json(config)
}

async fn epg(data: web::Data<AppState>) -> impl Responder {
    let stations_mutex = data.service.stations();
    let stations = &*stations_mutex.lock().unwrap();
    HttpResponse::Ok().json(stations)
}

async fn watch(req: HttpRequest) -> impl Responder {
    let id = req.match_info().get("id").unwrap();

    HttpResponse::Ok().body(format!("watch: {}", id))
}

// #[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    service: Arc<LocastService>,
    port: u16,
    station_scan: Mutex<bool>,
}

#[actix_web::main]
pub async fn start(services: Vec<Arc<LocastService>>, config: Arc<Config>) -> std::io::Result<()> {
    let servers: Vec<Server> = services
        .into_iter()
        .enumerate()
        .map(|(i, service)| {
            let port = config.port + i as u16;
            let bind_address = &config.bind_address;
            println!("Starting http server on http://{}:{}", bind_address, port);
            let app_state = web::Data::new(AppState {
                config: config.clone(),
                service: service.clone(),
                port: port,
                station_scan: Mutex::new(false),
            });
            HttpServer::new(move || {
                App::new()
                    .app_data(app_state.clone())
                    .route("/", web::get().to(device_xml))
                    .route("/device.xml", web::get().to(device_xml))
                    .route("/discover.json", web::get().to(discover))
                    .route("/lineup_status.json", web::get().to(lineup_status))
                    .route("/tuner.m3u", web::get().to(tuner_m3u))
                    .route("/lineup.json", web::get().to(lineup_json))
                    .route("/epg", web::get().to(epg))
                    .route("/config", web::get().to(show_config))
                    .service(web::resource("/watch/{id}").route(web::get().to(watch)))
            })
            .bind((bind_address.to_owned(), port))
            .unwrap()
            .run()
        })
        .collect();

    println!("Server started..");
    future::try_join_all(servers).await?;
    Ok(())
}

trait Or {
    /// Return `self` if it's not empty, otherwise `other`
    fn or<'a>(&'a self, other: &'a str) -> &str;
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
