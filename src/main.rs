#![recursion_limit = "256"]
extern crate chrono;
extern crate chrono_tz;
mod config;
mod credentials;
mod errors;
mod fcc_facilities;
mod http;
mod service;
mod utils;
use atty::Stream;
use chrono::Local;
use env_logger::Builder;
use itertools::Itertools;
use log::{info, warn, LevelFilter};
use service::multiplexer::Multiplexer;
use simple_error::SimpleError;
use std::sync::Arc;
use std::{env, io::Write};
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
#[actix_web::main]
async fn main() -> Result<(), SimpleError> {
    // Create a configuration struct that we'll pass along throughout the application
    let conf = match config::Config::from_args_and_file() {
        Ok(c) => Arc::new(c),
        Err(e) => panic!("{}", e),
    };

    // Log level 0 and 1 give info logging, but loglevel 1 adds HTTP logging.
    // Level 2 is debug and anything else defaults to trace.
    let log_level = match conf.verbose {
        0 | 1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    // Enable the RUST_BACKTRACE=1 env variable.
    if conf.rust_backtrace {
        env::set_var("RUST_BACKTRACE", "1");
    }

    let force_timestamps = conf.clone().force_timestamps;

    // Create the proper log format, but only prefix the date and level if we
    // have a tty, since we don't want to log date and level twice when using
    // syslog or something.
    Builder::new()
        .format(move |buf, record| {
            if atty::is(Stream::Stdout) || force_timestamps {
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

    info!("UUID: {}", conf.clone().uuid);

    // Login to locast and get credentials we pass around
    let credentials = Arc::new(credentials::LocastCredentials::new(conf.clone()).await);

    // Load FCC facilities
    let fcc_facilities = Arc::new(fcc_facilities::FCCFacilities::new(conf.clone()).await);

    // Create Locast Services
    let services = if let Some(zipcodes) = &conf.override_zipcodes {
        let services = zipcodes
            .into_iter()
            .map(|x| {
                service::LocastService::new(
                    conf.clone(),
                    credentials.clone(),
                    fcc_facilities.clone(),
                    Some(x.to_string()),
                )
            })
            .collect_vec();
        futures::future::join_all(services).await
    } else {
        vec![service::LocastService::new(conf.clone(), credentials, fcc_facilities, None).await]
    };

    // Create a multiplexer if necessary
    if conf.multiplex {
        if conf.remap {
            warn!("Channels will be remapped!")
        }
        let mp = vec![Multiplexer::new(services, conf.clone())];
        match http::start(mp, conf.clone()).await {
            Ok(()) => Ok(()),
            Err(_) => return Err(SimpleError::new("Failed to start servers")),
        }
    } else {
        match http::start(services, conf.clone()).await {
            Ok(()) => Ok(()),
            Err(_) => return Err(SimpleError::new("Failed to start servers")),
        }
    }
}
