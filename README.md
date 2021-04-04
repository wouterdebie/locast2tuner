# locast2tuner

This application provides an interface between locast.org and [Plex Media Server (PMS)](https://plex.tv) or [Emby](https://emby.media) by acting like a [HDHomerun](https://www.silicondust.com/) or an m3u Tuner and an XMLTV provider.

`locast2tuner` can imitate one or more digital tuners and provides geo cloaking across regions.

`locast2tuner` is a rewrite in Rust of [locast2dvr](https://github.com/wouterdebie/locast2dvr), which in turn is a rewrite of [locast2plex](https://github.com/tgorgdotcom/locast2plex). Thanks to the locast2plex developers for writing it and figuring out how to stitch things together!

I rewrote locast2plex to be able to more easily add functionality, use libraries wherever possible (like HTTP, m3u, starting multiple devices, etc), heavily document, generally write clean code and provide a better user experience (command line argument parsing, automatic download of FCC facilities, etc). And since python's GIL gave me a headache, I rewrote the whole thing in Rust.

Apart from the fact that everything is Rust now, the big difference between `locast2tuner` and `locast2dvr` is that `locast2tuner` does not require ffmpeg anymore. Actually, I completely dropped support for it and only implemented the `direct mode` that `locast2dvr` supports. Next to that, I removed a few debugging features (like --multiplex-debug), that don't seem to be used.

Even though this project started as a locast to PMS interface, it's more focused on integrating locast with Emby, since Emby provides a bit more functionality when it comes to Live TV and Tuner (like m3u tuners, XMLTV, etc).

## Features
- Override your location using zipcode or GPS coordinates
- Multiple digital tuners in a single server, either as separate servers or as one (multiplexing)
- SSDP for easy discovery of Tuner devices in PMS or Emby
- Acts like either a HDHomerun Tuner or m3u tuner
- Provides locast EPG information as an XMLTV guide

## TODO
This project isn't complete yet. It works, but there are a few things I'd like to get done:
- Code and user documentation: there isn't much documentation in the code yet and there is no user documentation at all.
- Packaging: I'd like to be able to supply binary packages for at least Ubuntu, MacOS and Windows. And of course a Docker container.
- Running locast2tuner as a daemon.
- Unit and integration tests.
- Upgrade mechanism.
- Since I'm pretty new to Rust, the code is probably not as idiomatic as it could be.
- Get `--logfile working`.
- Get `--override_location` working (maybe?)
- Currently a sync and async HTTP client is used and I might want to consolidate this. We might want to get rid of reqwest, since actix-web comes with a http client.
- Improve error handling
## Build prerequisites
- Rust 1.50.0 (`brew install rust` on MacOS)
- An active locast.org account with an active donation. Locast doesn't allow you to stream without a donation.


## Build
```sh
$ git clone https://github.com/wouterdebie/locast2tuner
$ cd locast2tuner
$ cargo build
```

## Install
Since there are no packages available yet, you'll end up with a binary in `./target/debug/locast2tuner`. You can copy this around.


## Usage
```
USAGE:
    locast2tuner [FLAGS] [OPTIONS]

FLAGS:
        --disable_station_cache    Disable stations cache
    -h, --help                     Prints help information
    -m, --multiplex                Multiplex devices
    -r, --remap                    Remap
    -s, --ssdp                     Enable SSDP
    -V, --version                  Prints version information

OPTIONS:
    -a, --bind <bind_address>                      Bind address (default: 127.0.0.1)
        --bytes_per_read <bytes_per_read>          Bytes per read(default: 1152000)
        --cache_timeout <cache_timeout>            Cache timeout (default: 3600)
    -c, --config <config>                            Config File
    -d, --days <days>                              Nr. of days to get EPG data for (default: 8)
        --device_firmware <device_firmware>          Device firmware (default: hdhomerun3_atsc)
        --device_model <device_model>              Device model (default: HDHR3-US)
        --device_version <device_version>          Device version (default: 20170612)
    -l, --logfile <logfile>                          Log file location
    -z, --override_zipcodes <override_zipcodes>    Override zipcodes
    -P, --password <password>                      Locast password
    -p, --port <port>                              Bind TCP port (default: 6077)
        --tuner_count <tuner_count>                Tuner count (default: 3)
    -U, --username <username>                      Locast username
    -v, --verbose <verbose>                        Verbosity (default: 0)
```

## Configuration
`locast2tuner` parameters can be specified as either command line arguments or in a configuration file that can be specified using that `--config` argument.

### Location overrides

By default `locast2tuner` uses your IP address to determine your location, but it also allows you to override the locast.org location you're creating a Tuner for:

- `override_zipcodes`, which takes a comma separated list of zipcodes as an argument. E.g. `--override_zipcodes 90210,55111` for Los Angeles and Minneapolis.

### <a name="multi_region"></a>Multi regions

`locast2tuner` allows starting multiple instances. This is done using the `override_zipcodes` option. A [file with all available locast regions](https://github.com/wouterdebie/locast2tuner/blob/main/regions) is included in the `locast2tuner` distribution.

When using multiple regions, `locast2tuner` will start multiple instances on TCP ports starting at the value that is specified with the `port` (or the default `6077`) argument and incremented by one and it will generate UUIDs for each tuner.

Note: PMS supports multiple devices, but does not support multiple Electronic Programming Guides (EPGs). Emby does support both. I personally use Emby since it allows for multiple EPGs.

### Usage in PMS or Emby

#### Tuners
`locast2tuner` can act as both a HDHomerun device or as an m3u tuner. Plex mainly supports HDHomerun, while Emby supports both. In case `locast2tuner` is used as an HDHomerun device it will copy the `mpegts` stream from locast to the Media server. When using `locast2tuner` as an m3u tuner, it will pass on the m3u from locast to the media server without any decoding.

- For use as a HDHomerun tuner, use `IP:PORT` (defaults to `127.0.0.1:6077`) to connect
- For use as an m3u tuner, use `http://IP:PORT/tuner.m3u` (defaults to `http://127.0.0.1:6077/tuner.m3u`) as the URL to connect.

#### EPG
`locast2tuner` also provides Electronic Programming Guide (EPG) information from locast. This is served in [XMLTV](http://wiki.xmltv.org/) format. Emby has support for XMLTV and can be used by adding `http://IP:PORT/epg.xml`  (defaults to `http://127.0.0.1:6077/epg.xml`) as an XMLTV TV Guide Data Provider.

### Multiplexing

`locast2tuner` normally starts an HTTP instance for each Tuner, starting at `port` (default `6077`). But with the option `--multiplex`, it will start a single HTTP interface multiplexing all Tuners through one interface for both streaming and EPG. Any channels that have the same call sign (like 4.1 ABC) will be deduped.

For example: if you use `--multiplex --override_zipcodes=90210,55111`, all channels from both zipcodes will be available, but multiplexed at `localhost:6077`.

Note: This type of multiplexing makes sense in Emby, since you can add a single tuner at `http://PORT:IP` or `http://PORT:IP/lineup.m3u` and a single EPG at `http://PORT:IP/epg.xml`
