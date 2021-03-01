#![recursion_limit = "256"]
extern crate chrono;
extern crate chrono_tz;
mod config;
mod credentials;
mod fcc_facilities;
mod http;
mod service;
mod utils;
mod xml_templates;
use service::LocastServiceArc;
use simple_error::SimpleError;
use std::sync::Arc;

fn main() -> Result<(), SimpleError> {
    // Get config from command line and config file
    // let config = match config::Config::from_args_and_file() {
    //     Ok(c) => c,
    //     Err(e) => return Err(e),
    // };

    let conf = Arc::new(config::Config::from_args_and_file()?);
    println!("{:?}", conf);

    // Login to locast and get credentials we pass around
    let credentials = Arc::new(credentials::LocastCredentials::new(conf.clone()));

    // Load FCC facilities
    let fcc_facilities = Arc::new(fcc_facilities::FCCFacilities::new(conf.clone()));

    // Create Locast Services
    let services: Vec<LocastServiceArc> = if let Some(zipcodes) = &conf.override_zipcodes {
        zipcodes.into_iter()
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

    match http::start::<LocastServiceArc>(services, conf.clone()) {
        Ok(()) => Ok(()),
        Err(_) => return Err(SimpleError::new("Failed to start servers")),
    }
}
