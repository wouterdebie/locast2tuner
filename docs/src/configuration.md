# Configuration

`locast2tuner` parameters can be specified as either command line arguments, as environment variables, or in a [TOML](https://github.com/toml-lang/toml) configuration file that can be specified using the `--config` argument.

> The precedence of parameters is **command line arguments** > **environment variables** > **config file**
## Command line arguments
In order to specify command line arguments, use `--argument value` (e.g. `--override_zipcodes 90210,30301`). Note that underscores are retained.

## Environment variables
Parameters can also be specified using environment variables. Environment variables are prefixed with `l2t_` and should be exported in the format `l2t_argument="value"` (e.g. `export l2t_override_zipcodes="90210,30301"`). Note that underscores and lowercase are retained.

For boolean flags (like `l2t_multiplex`), use `true` or `false` as values.

## Configuration file
The configuration file format is:

```toml
string_option = "<value1>"
flag = <true/false>
numerical_option = <number>
list_option = ["<value1>", "<value2>"]
```

Example:
```toml
username = "<Locast username>"
password = "<Locast password>"
verbose = 2
multiplex = true
override_zipcodes = ["85355", "90210"]
```

## Configuration options
Please have a look at the [example config file](https://raw.githubusercontent.com/wouterdebie/locast2tuner/main/assets/config.example)

Option | Description | Default
- | - | -
username _(required)_   | Locast.org username
password _(required)_   | Locast.org password
bind_address            | Address of the interface to bind to. To bind to all interfaces, use 0.0.0.0 | 127.0.0.1
cache_dir               | Cache data location | `$HOME/.locast2tuner`
cache_timeout           | How often (in seconds) the station cache is refreshed | 3600
days                    | Days of EPG data to fetch | 8, which is the maximum of data locast.org provides
device_firmware         | Device firmware that is reported to Plex or Emby | homerun3_atsc
device_model            | Device model that is reported to Plex or Emby | HDHR3-US
device_version          | Device version that is reported to Plex or Emby | 20170612
disable_station_cache   | Disable caching of station information. By default `locast2tuner` caches station information for an hour (see `cache_timeout`). By disabling the cache, every request for station information will lead to a call to locast.org. Normally you shouldn't have to disable the cache | false
disable_donation_check  | Disable the donation check. This doesn't mean you can watch without a donation, but the donation check fails for Locast Cares accounts | false
logfile                 | Log to a specific file | By default `locast2tuner` will not log to a file
multiplex               | Normally, when you override multiple zip codes, `locast2tuner` starts multiple instances (see "bind_address"), but with "multiplex = true", stations from multiple locations will be available through a single instance | false
no_tvc_guide_station    | Don't include `tvc_guide_station` in `tuner.m3u`. Having this field sometimes breaks things in Channels DVR. | false
override_cities         | Cities to override the location. Please see [locast.org](https://www.locast.org/dma) for a current map of the supported regions. This should be a pipe separated list with cities and states. E.g. `--override_cities "Los Angeles, CA\|Portland, OR"`| Unset. `locast2tuner` will use your external IP to determine your location
override_zipcodes       | Zip codes to override the location. Please see [locast.org](https://www.locast.org/dma) for a current map of the supported regions. This should be a comma separated list. E.g. `--override_zipcodes "90210,33101"`| Unset. `locast2tuner` will use your external IP to determine your location
port                    | TCP port to bind to. The default is 6077. In case you override muliple zip codes, `locast2tuner` will bind to multiple ports, starting at the number specified below (or default 6077). Any additional instance will bind to a port incremented by 1. E.g. if you override 3 zip codes, 3 instances will be started and bound to 6077, 6078 and 6079. In order to only use one instance, use `multiplex` | 6077
quiet                   | Don't output anything to the terminal | false
random_zipcode      | When `--override_cities` is used, `locast2tuner` looks up a list of valid zip codes for each city and will pick the first valid zip code, with `--random_zipcode` a random valid zip code for the city specified will be picked. | false
remap                   | Remap channel numbers when `multiplexing`. In case you override multiple zip codes, Emby and Plex will sort channels by channel number, which means channels from different locations might be intermingled. In order circumvent this, you can use "remap = true". This causes `locast2tuner` to rewrite the channel number based on the amount of instances there are. Locast will remap a "channel_number" to "channel_number + 100 * instance_number", where the instance_number starts at 0. E.g. you override 3 zip codes, then the channels from the first location will be untouched (since 100*0 == 0), the stations for the second location will start at 100 (e.g. 2.1 CBS becomes 102.1 CBS) and the stations for the third location will start at 200 (e.g. 13.2 WWFF becomes 213.2 WWFF). Note that `multiplex` has to be enabled! | false
remap_file              | File that can be used to do a custom remap. More info can be found [here](advanced/remapping.md). Note that `multiplex` has to be enabled! | Unset
skip_hls                | Instead of using hls.locastnet.org, use the proxy closer to the destination | false

rust_backtrace          | Enable RUST_BACKTRACE=1. In error logs, you might see "run with `RUST_BACKTRACE=1` environment variable to display a backtrace". Instead of adding the environment variable, you can enable this behavior with `rust_backtrace` | false
syslog                  | Log through syslogd | false
tuner_count             | The amount of tuners that is communicated to Plex. This will limit the amount of streams that Plex will Note that this is not a limitation in `locast2tuner` | 16
verbose                 | Verbosity. 0 = Info, 1 = Info + HTTP request lgos, 2 = Debug, 3 = Trace. In error logs, you might see "run with `RUST_BACKTRACE=1` environment variable to display a backtrace". Setting the verbosity to 2 or 3 will also include the backtrace | 0

## Displaying running config
You can display your running config (which could be a combination of a config file and command line parameters) by opening the `/config` path (e.g. `http://127.0.0.1:6077/config`). Normally the password is obfuscated, but if you add the query parameter `show_password` (e.g. `http://127.0.0.1:6077/config?showpass`), the password will become visible.
