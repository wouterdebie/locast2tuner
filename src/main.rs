#![recursion_limit = "256"]
extern crate chrono;
extern crate chrono_tz;
mod config;
mod credentials;
mod fcc_facilities;
mod http;
mod service;
mod utils;
use atty::Stream;
use chrono::Local;
use env_logger::Builder;
use log::{info, warn, LevelFilter};
use service::multiplexer::Multiplexer;
use simple_error::SimpleError;
use std::io::Write;
use std::sync::Arc;
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
fn main() -> Result<(), SimpleError> {
    let conf = Arc::new(config::Config::from_args_and_file()?);

    let log_level = match conf.verbose {
        0 | 1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    Builder::new()
        .format(|buf, record| {
            if atty::is(Stream::Stdout) {
                writeln!(
                    buf,
                    "{} [{}] - {}",
                    Local::now().format("%Y-%m-%dT%H:%M:%S"),
                    record.level(),
                    record.args()
                )
            } else {
                writeln!(buf, "[{}] - {}", record.level(), record.args())
            }
        })
        .filter(None, log_level)
        .init();

    info!(
        "locast2tuner {} on {} {} starting..",
        VERSION,
        sys_info::os_type().unwrap(),
        sys_info::os_release().unwrap()
    );

    info!("UUID: {}", conf.uuid);

    // Login to locast and get credentials we pass around
    let credentials = Arc::new(credentials::LocastCredentials::new(conf.clone()));

    // Load FCC facilities
    let fcc_facilities = Arc::new(fcc_facilities::FCCFacilities::new(conf.clone()));

    // Create Locast Services
    let services = if let Some(zipcodes) = &conf.override_zipcodes {
        zipcodes
            .into_iter()
            .map(|x| {
                service::LocastService::new(
                    conf.clone(),
                    credentials.clone(),
                    fcc_facilities.clone(),
                    Some(x.to_string()),
                )
            })
            .collect()
    } else {
        vec![service::LocastService::new(
            conf.clone(),
            credentials,
            fcc_facilities,
            None,
        )]
    };

    if conf.multiplex {
        if conf.remap {
            warn!("Channels will be remapped!")
        }
        let mp = vec![Multiplexer::new(services.clone(), conf.clone())];
        match http::start(mp, conf.clone()) {
            Ok(()) => Ok(()),
            Err(_) => return Err(SimpleError::new("Failed to start servers")),
        }
    } else {
        match http::start(services, conf.clone()) {
            Ok(()) => Ok(()),
            Err(_) => return Err(SimpleError::new("Failed to start servers")),
        }
    }
}
