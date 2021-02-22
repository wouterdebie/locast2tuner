mod config;
mod credentials;
mod fcc_facilities;
mod service;
mod utils;
use simple_error::SimpleError;

fn main() -> Result<(), SimpleError> {
    // Get config from command line and config file
    let config = &config::Config::from_args_and_file()?;
    println!("{:?}", config);

    // Login to locast and get credentials we pass around
    let credentials = &credentials::LocastCredentials::new(&config);

    // Load FCC facilities
    let fcc_facilities = &fcc_facilities::FCCFacilities::new(&config);

    // Create Locast Services
    let services: Vec<service::LocastService> = if let Some(o) = &config.override_zipcodes {
        o.into_iter()
            .map(|x| {
                service::LocastService::new(
                    config,
                    credentials,
                    fcc_facilities,
                    Some(x.to_string()),
                )
            })
            .collect()
    } else {
        vec![service::LocastService::new(
            config,
            credentials,
            fcc_facilities,
            None,
        )]
    };

    for service in services {
        println!("{}", service);
        service.stations();
    }
    Ok(())
}
