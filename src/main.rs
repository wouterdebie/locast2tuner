#![recursion_limit = "256"]
#[macro_use]
extern crate log;
mod config;
mod credentials;
mod errors;
mod fcc_facilities;
mod http;
mod logging;
mod service;
mod utils;
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;
use service::multiplexer::Multiplexer;
use simple_error::SimpleError;
use std::path::Path;
use std::sync::Arc;
use std::{env, fs};

const VERSION: &str = env!("CARGO_PKG_VERSION");

// Windows main
#[cfg(windows)]
fn main() {
    let _ = windows::run();
}

#[cfg(windows)]
mod windows {
    use std::{ffi::OsString, time::Duration};
    use windows_service::{
        define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher, Result,
    };

    const SERVICE_NAME: &str = "locast2tuner";
    const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

    pub fn run() -> Result<()> {
        winlog::try_register("locast2tuner Service Log").unwrap();

        // Register generated `ffi_service_main` with the system and start the service, blocking
        // this thread until the service is stopped.
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)
    }

    // Generate the windows service boilerplate.
    // The boilerplate contains the low-level service entry function (ffi_service_main) that parses
    // incoming service arguments into Vec<OsString> and passes them to user defined service
    // entry (my_service_main).
    define_windows_service!(ffi_service_main, my_service_main);

    pub fn my_service_main(_arguments: Vec<OsString>) {
        if let Err(_e) = run_service() {
            // Handle the error, by logging or something.
        }
    }

    pub fn run_service() -> Result<()> {
        // Define system service event handler that will be receiving service events.
        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                // Notifies a service to report its current status information to the service
                // control manager. Always return NoError even if not implemented.
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

                // Handle stop
                ServiceControl::Stop => {
                    sender.send_signal();
                    ServiceControlHandlerResult::NoError
                }

                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };

        // Register system service event handler.
        // The returned status handle should be used to report service status changes to the system.
        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

        // Tell the system that service is running
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::from_secs(60),
            process_id: None,
        })?;

        let mut rt = tokio::runtime::Runtime::new().unwrap();

        // let loglevel = rt.block_on(crate::determine_log_level());
        // std::env::set_var("RUST_LOG", &loglevel.to_string());
        winlog::init("locast2tuner Service Log").unwrap();

        let exitcode = rt.block_on(async move {
            match super::real_main(r).await {
                Ok(()) => {
                    info!("locast2tuner shutting down");
                    0
                }
                Err(e) => {
                    error!("Error in locast2tuner {}", e);
                    1
                }
            }
        });

        // loop {
        //     // Poll shutdown event.
        //     match shutdown_rx.recv_timeout(Duration::from_secs(1)) {
        //         // Break the loop either upon stop or channel disconnect
        //         Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,

        //         // Continue work if no events were received within the timeout
        //         Err(mpsc::RecvTimeoutError::Timeout) => (),
        //     };
        // }

        // Tell the system that service has stopped.
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(exitcode),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        Ok(())
    }
}

// Unix main
#[cfg(not(windows))]
#[actix_web::main]
async fn main() -> Result<(), SimpleError> {
    real_main().await
}

async fn real_main() -> Result<(), SimpleError> {
    // Create a configuration struct that we'll pass along throughout the application
    let conf = match config::Config::from_args_and_file() {
        Ok(c) => Arc::new(c),
        Err(e) => panic!("{}", e),
    };

    // Enable the RUST_BACKTRACE=1 env variable.
    if conf.rust_backtrace {
        env::set_var("RUST_BACKTRACE", "1");
    }

    // Log level 0 and 1 give info logging, but loglevel 1 adds HTTP logging.
    // Level 2 is debug and anything else defaults to trace.
    let log_level = match conf.verbose {
        0 | 1 => slog::Level::Info,
        2 => slog::Level::Debug,
        _ => slog::Level::Trace,
    };

    // Setup logging
    let logger = crate::logging::logger(log_level, &conf);
    let _scope_guard = slog_scope::set_global_logger(logger);
    let _log_guard = slog_stdlog::init().unwrap();

    info!(
        "locast2tuner {} on {} {} {}starting..",
        VERSION,
        sys_info::os_type().unwrap(),
        sys_info::os_release().unwrap(),
        running_in_container()
    );

    debug!("Main UUID: {}", conf.clone().uuid);

    info!("Consider sponsoring this project at https://github.com/sponsors/wouterdebie!");

    // Login to locast and get credentials we pass around
    let credentials = Arc::new(credentials::LocastCredentials::new(conf.clone()).await);

    // Load FCC facilities
    let fcc_facilities = Arc::new(fcc_facilities::FCCFacilities::new(conf.clone()).await);

    let zipcodes = if let Some(override_zipcodes) = conf.override_zipcodes.clone() {
        let x = override_zipcodes
            .into_iter()
            .map(|x| vec![x])
            .collect::<Vec<Vec<String>>>();
        Some(x)
    } else if let Some(cities) = &conf.override_cities {
        let z = cities
            .iter()
            .map(|c| match zip_codes_plus::by_city(c) {
                Some(z) => z
                    .iter()
                    .filter(|r| matches!(r.zip_code_type, zip_codes_plus::Type::Standard))
                    .map(|r| r.zip_code.to_string())
                    .collect::<Vec<String>>(),
                None => panic!("Unknown city: {}", c),
            })
            .collect();

        Some(z)
    } else {
        None
    };

    // Create Locast Services
    let services = if let Some(zipcodes) = zipcodes {
        let services = zipcodes
            .into_iter()
            .map(|mut z| {
                if conf.random_zipcode {
                    z.shuffle(&mut thread_rng())
                }

                service::LocastService::new(
                    conf.clone(),
                    credentials.clone(),
                    fcc_facilities.clone(),
                    Some(z),
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
            warn!("Channels will be remapped!");
        }
        let mp = vec![Multiplexer::new(services, conf.clone())];
        match http::start(mp, conf.clone()).await {
            Ok(()) => Ok(()),
            Err(_) => Err(SimpleError::new("Failed to start servers")),
        }
    } else {
        match http::start(services, conf.clone()).await {
            Ok(()) => Ok(()),
            Err(_) => Err(SimpleError::new("Failed to start servers")),
        }
    }
}

#[cfg(not(windows))]
fn running_in_container() -> &'static str {
    // Check to see if running in a container
    let cgroup = Path::new("/proc/1/cgroup");
    if cgroup.exists() {
        let info: String = fs::read_to_string(cgroup).unwrap().parse().unwrap();
        if info.contains("docker") || info.contains("lxc") {
            "(Docker) "
        } else {
            ""
        }
    } else {
        ""
    }
}

#[cfg(windows)]
fn running_in_container() -> &'static str {
    ""
}
