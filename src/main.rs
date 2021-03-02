#![recursion_limit = "256"]
extern crate chrono;
extern crate chrono_tz;
mod config;
mod credentials;
mod fcc_facilities;
mod http;
mod multiplexer;
mod service;
mod utils;
mod xml_templates;
use simple_error::SimpleError;
use std::sync::Arc;

fn main() -> Result<(), SimpleError> {
    let conf = Arc::new(config::Config::from_args_and_file()?);
    println!("{:?}", conf);

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

    let mp = vec![multiplexer::Multiplexer::new(
        services.clone(),
        conf.clone(),
    )];

    if conf.multiplex {
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
