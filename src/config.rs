use clap_conf::*;
use serde::Serialize;
use simple_error::SimpleError;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;
#[derive(Default, Debug, Serialize, Clone)]
pub struct Config {
    pub logfile: Option<String>,
    pub bind_address: String,
    pub cache_directory: PathBuf,
    pub cache_timeout: u64,
    pub days: u8,
    pub device_firmware: String,
    pub device_model: String,
    pub device_version: String,
    pub disable_station_cache: bool,
    pub multiplex: bool,
    pub override_zipcodes: Option<Vec<String>>,
    pub password: String,
    pub port: u16,
    pub quiet: bool,
    pub remap: bool,
    pub rust_backtrace: bool,
    pub ssdp: bool,
    pub syslog: bool,
    pub tuner_count: u8,
    pub username: String,
    #[serde(skip_serializing)]
    pub uuid: String,
    pub verbose: u8,
}
impl Config {
    pub fn from_args_and_file() -> Result<Config, SimpleError> {
        let clap = clap_app!(
            locast2tuner=>
                (version: crate_version!())
                (author: "Wouter de Bie")
                (about: "Locast to tuner")
                (@arg bind_address: -b --bind_address +takes_value "Bind address (default: 127.0.0.1)")
                (@arg cache_dir: --cache_dir +takes_value "Cache directory (default: $HOME/.locast2tuner)")
                (@arg cache_timeout: --cache_timeout +takes_value "Cache timeout (default: 3600)")
                (@arg config: -c --config +takes_value "Config File") //allow clap_conf config loader to work
                (@arg days: -d --days +takes_value "Nr. of days to get EPG data for (default: 8)")
                (@arg device_firmware: --device_firmware +takes_value "Device firmware (default: hdhomerun3_atsc)")
                (@arg device_model: --device_model +takes_value "Device model (default: HDHR3-US)")
                (@arg device_version: --device_version +takes_value "Device version (default: 20170612)")
                (@arg disable_station_cache: --disable_station_cache "Disable stations cache")
                (@arg multiplex: -m --multiplex "Multiplex devices")
                (@arg override_zipcodes: -z --override_zipcodes +takes_value "Override zipcodes")
                (@arg password: -P --password +takes_value "Locast password")
                (@arg port: -p --port +takes_value "Bind TCP port (default: 6077)")
                (@arg remap: -r --remap "Remap channels when multiplexed")
                (@arg rust_backtrace: --rust_backtrace "Enable RUST_BACKTRACE=1")
                (@arg ssdp: -s --ssdp "Enable SSDP")
                (@arg syslog: --syslog "Log to syslogd")
                (@arg quiet: --quiet "Don't log to terminal")
                (@arg tuner_count: --tuner_count +takes_value "Tuner count (default: 3)")
                (@arg username: -U --username +takes_value "Locast username")
                (@arg verbose: -v --verbose +takes_value "Verbosity (default: 0)")
                (@arg logfile: -l --logfile +takes_value "Log file location")

        )
        .get_matches();

        let mut conf = Self::default();
        let cfg = clap_conf::with_toml_env(&clap, &["/etc/locast2tuner/config"]);
        conf.username = cfg
            .grab()
            .arg("username")
            .conf("username")
            .done()
            .expect("Username required");
        conf.password = cfg
            .grab()
            .arg("password")
            .conf("password")
            .done()
            .expect("Password required");

        conf.bind_address = cfg
            .grab()
            .arg("bind_address")
            .conf("bind_address")
            .def("127.0.0.1");

        conf.port = cfg.grab().arg("port").conf("port").t_def::<u16>(6077);
        conf.verbose = cfg.grab().arg("verbose").conf("verbose").t_def::<u8>(0);
        conf.multiplex =
            cfg.bool_flag("multiplex", Filter::Arg) || cfg.bool_flag("multiplex", Filter::Conf);

        // First check if there's a comma-separated list from the command line
        conf.override_zipcodes = match cfg.grab().arg("override_zipcodes").done() {
            Some(o) => Some(o.split(',').map(|x| x.to_string()).collect()),
            None => {
                match cfg.grab().conf("override_zipcodes").done() {
                    // Overrides are defined as a regular string (old format)
                    Some(o) => Some(o.split(',').map(|x| x.to_string()).collect()),
                    // Overrides are defined as an arra (new format)
                    None => match cfg.grab_multi().conf("override_zipcodes").done() {
                        Some(o) => Some(o.collect()),
                        None => None,
                    },
                }
            }
        };

        conf.tuner_count = cfg
            .grab()
            .arg("tuner_count")
            .conf("tuner_count")
            .t_def::<u8>(3);

        conf.device_model = cfg
            .grab()
            .arg("device_model")
            .conf("device_model")
            .def("HDHR3-US");

        conf.device_firmware = cfg
            .grab()
            .arg("device_firmware")
            .conf("device_firmware")
            .def("hdhomerun3_atsc");

        conf.device_version = cfg
            .grab()
            .arg("device_version")
            .conf("device_version")
            .def("20170612");

        conf.disable_station_cache = cfg.bool_flag("disable_station_cache", Filter::Arg)
            || cfg.bool_flag("disable_station_cache", Filter::Conf);

        conf.syslog = cfg.bool_flag("syslog", Filter::Arg) || cfg.bool_flag("syslog", Filter::Conf);
        conf.quiet = cfg.bool_flag("quiet", Filter::Arg) || cfg.bool_flag("quiet", Filter::Conf);

        conf.cache_timeout = cfg
            .grab()
            .arg("cache_timeout")
            .conf("cache_timeout")
            .t_def::<u64>(3600);

        conf.days = cfg.grab().arg("days").conf("days").t_def::<u8>(8);

        conf.remap = cfg.bool_flag("remap", Filter::Arg) || cfg.bool_flag("remap", Filter::Conf);
        conf.rust_backtrace = cfg.bool_flag("rust_backtrace", Filter::Arg)
            || cfg.bool_flag("rust_backtrace", Filter::Conf);

        conf.logfile = cfg.grab().arg("logfile").conf("logfile").done();

        let default_cache_dir = dirs::home_dir().unwrap().join(Path::new(".locast2tuner"));

        let cache_directory_name = cfg
            .grab()
            .arg("cache_dir")
            .conf("cache_dir")
            .def(default_cache_dir.to_str().unwrap());

        let cache_directory = create_cache_directory(cache_directory_name);

        conf.uuid = load_uuid(&cache_directory).unwrap();

        conf.cache_directory = cache_directory;
        Ok(conf)
    }
}

// Create the cache directory
fn create_cache_directory(name: String) -> PathBuf {
    let cache_dir = Path::new(&name).to_path_buf();
    if !cache_dir.exists() {
        fs::create_dir(cache_dir.as_path())
            .expect(&format!("Unable to create directory {:?}", cache_dir)[..]);
    }
    cache_dir
}

// Load the UUID from cache directory if exists
fn load_uuid(cache_directory: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let uid_file = cache_directory.join(Path::new("uuid"));
    let uuid = match uid_file.exists() {
        true => fs::read_to_string(uid_file)?,
        false => generate_and_store_uid(uid_file),
    };

    Ok(uuid)
}

// Generate UUID and store it in the supplied path
fn generate_and_store_uid(path: PathBuf) -> String {
    let new_uuid = Uuid::new_v4().to_string();

    File::create(&path)
        .unwrap()
        .write_all((&new_uuid[..]).as_bytes())
        .expect("Unable to write uuid file");

    new_uuid
}
