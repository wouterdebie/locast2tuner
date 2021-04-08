# locast2tuner

This application provides an interface between locast.org and [Plex Media Server (PMS)](https://plex.tv) or [Emby](https://emby.media) by acting like an [HDHomerun](https://www.silicondust.com/) or an m3u Tuner and an XMLTV provider.

`locast2tuner` can imitate one or more digital tuners and provides geo cloaking across regions.

`locast2tuner` is a rewrite in Rust of [locast2dvr](https://github.com/wouterdebie/locast2dvr), which in turn is a rewrite of [locast2plex](https://github.com/tgorgdotcom/locast2plex). Thanks to the locast2plex developers for writing it and figuring out how to stitch things together!

I rewrote locast2plex to be able to more easily add functionality, use libraries wherever possible (like HTTP, m3u, starting multiple devices, etc), heavily document, generally write clean code, and provide a better user experience (command line argument parsing, automatic download of FCC facilities, etc). And since python's GIL gave me a headache, I rewrote the whole thing in Rust.

Apart from the fact that everything is Rust now, the big difference between `locast2tuner` and `locast2dvr` is that `locast2tuner` does not require ffmpeg anymore. Actually, I completely dropped support for it and only implemented the `direct mode` that `locast2dvr` supports. Next to that, I removed a few debugging features (like --multiplex-debug), that don't seem to be used.

Even though this project started as a locast to PMS interface, it's more focused on integrating locast with Emby, since Emby provides a bit more functionality when it comes to Live TV and Tuner (like m3u tuners, XMLTV, etc).

## Features
- Override your location using ZIP code or GPS coordinates
- Multiple digital tuners in a single server, either as separate servers or as one (multiplexing)
- Acts like either an HDHomerun Tuner or m3u tuner
- Provides locast EPG information as an XMLTV guide

## TODO
This project isn't complete yet. It works, but there are a few things I'd like to get done. These can be found on the [Issues page](https://github.com/wouterdebie/locast2tuner/issues)

## Getting started
Before you get started with installing and running locast2tuner, make sure you have an active [locast.org](https://locast.org) account with an active donation.
### Ubuntu/Debian
Ubuntu/Debian packages are available:
```sh
# Add the PPA key
$ curl -s "https://wouterdebie.github.io/ppa/KEY.gpg" | sudo apt-key add -
$ sudo curl -o /etc/apt/sources.list.d/locast2tuner.list "https://wouterdebie.github.io/ppa/sources.list"
$ sudo apt update
$ sudo apt install locast2tuner
```

Create a config file in `/etc/locast2tuner/config.ini` and enable and start the service:

```sh
$ sudo systemctl enable locast2tuner
$ sudo systemctl start locast2tuner
```

### Docker
A Docker image is available from `ghcr.io/wouterdebie/locast2dvr:latest` and is built from this [Dockerfile](https://github.com/wouterdebie/locast2tuner/blob/main/assets/docker/Dockerfile).

To run:
```sh
# Create a config directory (e.g. $HOME/.locast2tuner) and copy the example file in there:
$ mkdir $HOME/.locast2tuner
$ curl -o $HOME/.locast2tuner/config.ini https://github.com/wouterdebie/locast2tuner/blob/main/assets/config.ini.example
# ... edit the file ...
$ docker pull ghcr.io/wouterdebie/locast2dvr:latest
$ docker run -p 6077:6077 -v $HOME/.locast2tuner/:/app/config --name locast2tuner -d ghcr.io/wouterdebie/locast2dvr:latest
```

If you'd like to use `docker-compose` you can use the sample [docker-compose.yml](https://github.com/wouterdebie/locast2tuner/blob/main/assets/docker/docker-compose.yml).

### Other
For other operating systems, you will have to compile locast2tuner from source. The only build requirement `locast2tuner` has is [Rust](https://www.rust-lang.org/) 1.50.0+.

#### Installing dependencies
- MacOS: `brew install rust`
- Linux: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

#### Building
```sh
$ git clone https://github.com/wouterdebie/locast2tuner
$ cd locast2tuner
$ cargo build --release
```

#### Installing
You'll end up with a binary in `./target/release/locast2tuner`. You can copy this to the directory of your choosing (`/usr/local/bin` is a good place to start).

## Usage
```
USAGE:
    locast2tuner [FLAGS] [OPTIONS]

FLAGS:
        --disable_station_cache    Disable stations cache
    -h, --help                     Prints help information
    -m, --multiplex                Multiplex devices
    -r, --remap                    Remap channels when multiplexed
    -s, --ssdp                     Enable SSDP
    -V, --version                  Prints version information

OPTIONS:
    -b, --bind_address <bind_address>              Bind address (default: 127.0.0.1)
        --cache_dir <cache_dir>                    Cache directory (default: $HOME/.locast2tuner)
        --cache_timeout <cache_timeout>            Cache timeout (default: 3600)
    -c, --config <config>                          Config File
    -d, --days <days>                              Nr. of days to get EPG data for (default: 8)
        --device_firmware <device_firmware>        Device firmware (default: hdhomerun3_atsc)
        --device_model <device_model>              Device model (default: HDHR3-US)
        --device_version <device_version>          Device version (default: 20170612)
    -z, --override_zipcodes <override_zipcodes>    Override ZIP codes
    -P, --password <password>                      Locast password
    -p, --port <port>                              Bind TCP port (default: 6077)
        --tuner_count <tuner_count>                Tuner count (default: 3)
    -U, --username <username>                      Locast username
    -v, --verbose <verbose>                        Verbosity (default: 0)
```

## Configuration
`locast2tuner` parameters can be specified as either command line arguments or in a configuration file that can be specified using the `--config` argument.

The configuration file format is:

```sh
option1="<value1>"
option2="<value2>"
```

Example:
```sh
username="<Locast username>"
password="<Locast password>"
```

See [assets/config.ini.example](https://github.com/wouterdebie/locast2tuner/blob/main/assets/config.ini.example) for more.
### Location overrides

By default `locast2tuner` uses your IP address to determine your location, but it also allows you to override the locast.org location you're creating a Tuner for:

- `override_zipcodes`, which takes a comma separated list of ZIP codes as an argument. E.g. `--override_zipcodes 90210,55111` for Los Angeles and Minneapolis.

### <a name="multi_region"></a>Multi regions

`locast2tuner` allows starting multiple instances. This is done using the `override_zipcodes` option. A [file with all available locast regions](https://github.com/wouterdebie/locast2tuner/blob/main/regions) is included in the `locast2tuner` distribution.

When using multiple regions, `locast2tuner` will start multiple instances on TCP ports starting at the value that is specified with the `port` (or the default `6077`) argument and incremented by one and it will generate UUIDs for each tuner.

Note: PMS supports multiple devices, but does not support multiple Electronic Programming Guides (EPGs). Emby supports both. I personally use Emby since it allows for multiple EPGs.

### Usage in PMS or Emby

#### Tuners
`locast2tuner` can act as both a HDHomerun device or as an m3u tuner. Plex mainly supports HDHomerun, while Emby supports both. In case `locast2tuner` is used as an HDHomerun device it will copy the `mpegts` stream from locast to the Media server. When using `locast2tuner` as an m3u tuner, it will pass on the m3u from locast to the media server without any decoding.

- For use as a HDHomerun tuner, use `IP:PORT` (defaults to `127.0.0.1:6077`) to connect
- For use as an m3u tuner, use `http://IP:PORT/tuner.m3u` (defaults to `http://127.0.0.1:6077/tuner.m3u`) as the URL to connect.

#### EPG
`locast2tuner` also provides Electronic Programming Guide (EPG) information from locast. This is served in [XMLTV](http://wiki.xmltv.org/) format. Emby and PMS both have support for XMLTV which can be used by adding `http://IP:PORT/epg.xml`  (defaults to `http://127.0.0.1:6077/epg.xml`) as an XMLTV TV Guide Data Provider.

### Multiplexing

`locast2tuner` normally starts an HTTP instance for each Tuner, starting at `port` (default `6077`). But with the option `--multiplex`, it will start a single HTTP interface multiplexing all Tuners through one interface for both streaming and EPG. Any channels that have the same call sign (like 4.1 ABC) will be deduped.

For example: if you use `--multiplex --override_zipcodes=90210,55111`, all channels from both ZIP codes will be available, but multiplexed at `localhost:6077`.

Note: This type of multiplexing makes sense in Emby, since you can add a single tuner at `http://PORT:IP` or `http://PORT:IP/lineup.m3u` and a single EPG at `http://PORT:IP/epg.xml`


## Submitting bugs or feature requests
### Bugs
When you encounter a bug, please use [Github Issues](https://github.com/wouterdebie/locast2tuner/issues):
- Add a detailed description of the issue you're experiencing.
- Explain what steps can be taken to reproduce the issue.
- If possible, add an excerpt of the log file that shows the error.
- Add a copy of your config. You can get a copy of your running config by opening `/config` in a browser (e.g [http://localhost:6077/config](http://localhost:6077/config)). This will not expose your locast password. If you can't access `/config`, please add your `config.ini` *without* your password.
- Before submitting, mark the issue as a "Bug".

### Feature requests
When you have a features you'd like to see added, please use [Github Issues](https://github.com/wouterdebie/locast2tuner/issues) and mark the issue as an "Enhancement".

### Pull requests
If you're so awesome that you want to fix bugs or add features yourself, please fork this repository, code, and create a [Pull Request](https://docs.github.com/en/github/collaborating-with-issues-and-pull-requests/about-pull-requests). Please [squash your commits](https://www.git-tower.com/learn/git/faq/git-squash/) into a single commit before sending the pull request.
