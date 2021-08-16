# locast2tuner

[![Join the chat at https://gitter.im/wouterdebie/locast2tuner](https://badges.gitter.im/wouterdebie/locast2tuner.svg)](https://gitter.im/wouterdebie/locast2tuner?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge) ![build status](https://github.com/wouterdebie/locast2tuner/actions/workflows/release.yml/badge.svg) ![GitHub tag (latest SemVer)](https://img.shields.io/github/v/tag/wouterdebie/locast2tuner)


This application provides an interface between locast.org and [Plex Media Server (PMS)](https://plex.tv) or [Emby](https://emby.media) by acting like an [HDHomerun](https://www.silicondust.com/) or an m3u tuner and an XMLTV provider.

`locast2tuner` can imitate one or more digital tuners and provides geo cloaking across regions.

# Table of Contents
<!--ts-->
   * [locast2tuner](#locast2tuner)
   * [Table of Contents](#table-of-contents)
   * [Features](#features)
   * [Background](#background)
   * [Getting started](#getting-started)
      * [Ubuntu/Debian](#ubuntudebian)
      * [MacOS](#macos)
      * [Docker](#docker)
         * [Docker Compose (<em>recommended</em>)](#docker-compose-recommended)
            * [Running](#running)
            * [Upgrading](#upgrading)
         * [Docker on Unraid](#docker-on-unraid)
            * [Running using docker-compose](#running-using-docker-compose)
            * [Running using docker](#running-using-docker)
         * [Manual Docker](#manual-docker)
            * [Running](#running-1)
            * [Upgrading](#upgrading-1)
      * [Building from source](#building-from-source)
         * [Installing dependencies](#installing-dependencies)
         * [Building](#building)
         * [Installing](#installing)
   * [Usage](#usage)
   * [Quickstart guides for Plex and Emby](#quickstart-guides-for-plex-and-emby)
   * [Configuration](#configuration)
      * [Displaying running config](#displaying-running-config)
      * [Location overrides](#location-overrides)
      * [Multi regions](#multi-regions)
         * [Tuner emulation](#tuner-emulation)
         * [EPG](#epg)
      * [Multiplexing](#multiplexing)
      * [Remapping](#remapping)
      * [Logging](#logging)
   * [TODO](#todo)
   * [Submitting bugs or feature requests](#submitting-bugs-or-feature-requests)
      * [Bugs](#bugs)
      * [Feature requests](#feature-requests)
      * [Pull requests](#pull-requests)
<!--te-->

# Features
- Override your location using ZIP code or GPS coordinates
- Multiple digital tuners in a single server, either as separate servers or as one (multiplexing)
- Acts like either an HDHomerun or an m3u tuner
- Provides locast EPG information as an XMLTV guide
# Background
`locast2tuner` is a rewrite in Rust of [locast2dvr](https://github.com/wouterdebie/locast2dvr), which in turn is a rewrite of [locast2plex](https://github.com/tgorgdotcom/locast2plex). Thanks to the locast2plex developers for writing it and figuring out how to stitch things together!

I rewrote locast2plex to be able to more easily add functionality, use libraries wherever possible (like HTTP, m3u, starting multiple devices, etc), heavily document, generally write clean code, and provide a better user experience (command line argument parsing, automatic download of FCC facilities, etc). And since python's GIL gave me a headache, I rewrote the whole thing in Rust.

Apart from the fact that everything is Rust now, the big difference between `locast2tuner` and `locast2dvr` is that `locast2tuner` does not require ffmpeg anymore. Actually, I completely dropped support for it and only implemented the `direct mode` that `locast2dvr` supports. Next to that, I removed a few debugging features (like --multiplex-debug), that don't seem to be used.

Even though this project started as a locast to PMS interface, it's more focused on integrating locast with Emby, since Emby provides a bit more functionality when it comes to Live TV and Tuner (like m3u tuners, XMLTV, etc).

# Getting started
Before you get started with installing and running locast2tuner, make sure you have an active [locast.org](https://locast.org) account with an active donation.
## Ubuntu/Debian
Ubuntu/Debian packages are available for both amd64 and arm7 (Raspbian):
```sh
# Add the PPA key
curl -s "https://wouterdebie.github.io/ppa/KEY.gpg" | sudo apt-key add -
# Add the locast2tuner repository
sudo curl -o /etc/apt/sources.list.d/locast2tuner.list "https://wouterdebie.github.io/ppa/sources.list"
sudo apt update
# Install locast2tuner
sudo apt install locast2tuner
```

Create a config file. Don't forget to edit the config file!
```sh
sudo cp /etc/locast2tuner/config.example /etc/locast2tuner/config
# .. edit the config /etc/locast2tuner/config (e.g nano /etc/locast2tuner/config) ..
```
Finally, enable and start the service:

```sh
sudo systemctl enable locast2tuner
sudo systemctl start locast2tuner
```
## MacOS
A MacOS package is available though [Homebrew](https://brew.sh/):
```sh
# Install from homebrew
brew tap wouterdebie/repo
brew install locast2tuner
# Get the sample config
curl -o locast2tuner.config https://raw.githubusercontent.com/wouterdebie/locast2tuner/main/assets/config.example
# .. edit locast2tuner.config ..
# Run locast2tuner
locast2tuner --config locast2tuner.config
```

## Docker
A Docker image is available from `ghcr.io/wouterdebie/locast2tuner:latest` and is built from this [Dockerfile](https://github.com/wouterdebie/locast2tuner/blob/main/assets/docker/Dockerfile).

You will need a [configuration](#configuration) file with a minimum of your Locast account settings in it:

```bash
# Create a config directory (e.g. $HOME/.locast2tuner) and copy the example file in there:
mkdir $HOME/.locast2tuner
# Download a sample config file in $HOME/.locast2tuner/config
curl -o $HOME/.locast2tuner/config https://raw.githubusercontent.com/wouterdebie/locast2tuner/main/assets/config.example
# ... edit the file to match your settings ...
```

>The Docker image has the path of the configuration file hardcoded to `/app/config/config` inside the container, so make sure the volume mappings are correct.

>locast2tuner requires Unix style line endings (i.e. LF) in the config file. This means that in Windows you have to make sure that the configuration file is saved in the right format. Either use an editor that allows you to save with Unix style line endings (like Notepad++ or VS Code) or you can run `dos2unix config` to remove DOS/Windows (i.e. CRLF) line endings prior to launching the container.

### Docker Compose (_recommended_)

If you'd like to use Docker Compose  you can use the sample [docker-compose.yml](https://github.com/wouterdebie/locast2tuner/blob/main/assets/docker/docker-compose.yml) and edit it to match your settings.
#### Running

```sh
# Get the sample docker-compose.yml
curl -o docker-compose.yml https://raw.githubusercontent.com/wouterdebie/locast2tuner/main/assets/docker/docker-compose.yml
# .. edit docker-compose.yml to match your settings ..
# Start the Docker container
docker-compose up -d
```

#### Upgrading

```sh
# Pull any new versions of images referenced in docker-compose.yml
docker-compose pull
# Restart any containers in `docker-compose.yml` with newly pulled images
docker-compose up -d
```

### Docker on Unraid
Thanks to [RandomNinjaAtk](https://github.com/RandomNinjaAtk) there's a specific x86-64 Docker image for Unraid that is built using this [Dockerfile.unraid](https://github.com/wouterdebie/locast2tuner/blob/main/assets/docker/Dockerfile.unraid). The big difference with the Docker image described above is that this image is configured using environment variables, rather than through an externally mounted configuration file.

The container image is configured with the following parameters passed at runtime:

| Parameter | Function |
| ---- | --- |
| `-p 6077` | The port for the tuner access |
| `-e L2TUSER=username` | Locast Username |
| `-e L2TPASS=password` | Locast Password |
| `-e L2TZIP=#####,#####` | Locast Zipcodes, zipcode in format: #####,#####,##### |
| `-e multiplex=true` | Enables multiplexing |

#### Running using docker-compose
Compatible with docker-compose v2 schemas.
```
version: "2.1"
services:
  locast2tuner:
    image: ghcr.io/wouterdebie/locast2tuner-unraid:latest
    container_name: locast2tuner
    environment:
      - L2TUSER=username
      - L2TPASS=password
      - L2TZIP=#####,#####
      - multiplex=true
    ports:
      - 6077:6077
    restart: unless-stopped
```

#### Running using docker
```
docker create \
  --name=locast2tuner \
  -p 6077 \
  -e L2TUSER=username \
  -e L2TPASS=password \
  -e L2TZIP=#####,##### \
  -e multiplex=true \
  ghcr.io/wouterdebie/locast2tuner-unraid:latest
```

### Manual Docker

#### Running
```sh
# Pull latest locast2tuner image
docker pull ghcr.io/wouterdebie/locast2tuner:latest
# Start the container
docker run -p 6077:6077 -v $HOME/.locast2tuner/:/app/config --name locast2tuner -d ghcr.io/wouterdebie/locast2tuner:latest
```
#### Upgrading
```sh
# Pull the latest version of the locast2tuner container image
docker pull ghcr.io/wouterdebie/locast2tuner:latest
# Stop the existing container
docker stop locast2tuner
# Remove the existing container
docker rm locast2tuner
# Start the container
docker run -p 6077:6077 -v $HOME/.locast2tuner/:/app/config --name locast2tuner -d ghcr.io/wouterdebie/locast2tuner:latest
```

## Building from source
The only build requirement `locast2tuner` has is [Rust](https://www.rust-lang.org/) 1.50.0+.

### Installing dependencies
- MacOS: `brew install rust`
- Linux: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### Building
```sh
git clone https://github.com/wouterdebie/locast2tuner
cd locast2tuner
cargo build --release
```

### Installing
You'll end up with a binary in `./target/release/locast2tuner`. You can copy this to the directory of your choosing (`/usr/local/bin` is a good place to start).

# Usage
For usage options, please run `locast2tuner -h`.
# Quickstart guides for Plex and Emby
Detailed instructions are available for integrating `locast2tuner` with both [Plex](docs/01_plex.md) and [Emby](docs/02_emby.md). Make sure to check out the detailed configuration below.
# Configuration
`locast2tuner` parameters can be specified as either command line arguments or in a [TOML](https://github.com/toml-lang/toml) configuration file that can be specified using the `--config` argument.

The configuration file format is:

```sh
string_option = "<value1>"
flag = <true/false>
numerical_option = <number>
list_option = ["<value1>", "<value2>"]
```

Example:
```sh
username = "<Locast username>"
password = "<Locast password>"
verbose = 2
multiplex = true
override_zipcodes = ["85355", "90210"]
```

See [assets/config.example](https://raw.githubusercontent.com/wouterdebie/locast2tuner/main/assets/config.example) for more information and a description of each option.

## Displaying running config
You can display your running config (which could be a combination of a config file and command line parameters) by opening the `/config` path (e.g. `http://127.0.0.1:6077/config`). Normally the password is obfuscated, but if you add the query parameter `show_password` (e.g. `http://127.0.0.1:6077/config?showpass`), the password will become visible.

## Location overrides
By default `locast2tuner` uses your IP address to determine your location, but it also allows you to override the locast.org location you're creating a Tuner for:

- `override_zipcodes`, which takes a comma separated list of ZIP codes as an argument. E.g. `--override_zipcodes 90210,55111` for Los Angeles and Minneapolis.

## Multi regions
`locast2tuner` allows starting multiple instances. This is done using the `override_zipcodes` option. A [file with all available locast regions](https://github.com/wouterdebie/locast2tuner/blob/main/assets/regions) is included in the `locast2tuner` distribution.

When using multiple regions, `locast2tuner` will start multiple instances on TCP ports starting at the value that is specified with the `port` (or the default `6077`) argument and incremented by one and it will generate UUIDs for each tuner.

Note: PMS supports multiple devices, but does not support multiple Electronic Programming Guides (EPGs). Emby supports both. I personally use Emby since it allows for multiple EPGs.

### Tuner emulation
`locast2tuner` can act as both a HDHomerun device or as an m3u tuner. Plex mainly supports HDHomerun, while Emby supports both. In case `locast2tuner` is used as an HDHomerun device it will copy the `mpegts` stream from locast to the Media server. When using `locast2tuner` as an m3u tuner, it will pass on the m3u from locast to the media server without any stream interference. This means that the media server will directly connect to
the stream.

- For use as a HDHomerun tuner, use `IP:PORT` (defaults to `127.0.0.1:6077`) to connect
- For use as an m3u tuner, use `http://IP:PORT/tuner.m3u` (defaults to `http://127.0.0.1:6077/tuner.m3u`) as the URL to connect.

### EPG
`locast2tuner` also provides Electronic Programming Guide (EPG) information from locast.org. This is served in the [XMLTV](http://wiki.xmltv.org/) format. Emby and PMS both have support for XMLTV which can be used by adding `http://IP:PORT/epg.xml`  (defaults to `http://127.0.0.1:6077/epg.xml`) as an XMLTV TV Guide Data Provider.

## Multiplexing

`locast2tuner` normally starts an HTTP instance for each Tuner, starting at `port` (default `6077`). But with the option `--multiplex`, it will start a single HTTP interface multiplexing all Tuners through one interface for both streaming and EPG.

For example: if you use `--multiplex --override_zipcodes=90210,55111`, all channels from both ZIP codes will be available, but multiplexed at `localhost:6077`.

Note: This type of multiplexing makes sense in Emby, since you can add a single tuner at `http://PORT:IP` or `http://PORT:IP/lineup.m3u` and a single EPG at `http://PORT:IP/epg.xml`

## Remapping
In case you override multiple zip codes, Emby and Plex will sort channels by channel number, which means channels from different locations might be intermingled. In order circumvent this, you can remap channels.  `locast2tuner` offers two ways of remapping channels.  Note that these two options are mutually exclusive, but both can appear in a config file. If both appear, then the `--remap` option takes precedence.

The easiest way is to use `--remap` option. This causes locast2tuner to rewrite the channel number based on the amount of instances there are. Locast will remap a "channel_number" to "channel_number + 100 * instance_number", where the instance_number starts at 0. E.g. you override 3 zip codes, then the channels from the first location will be untouched (since 100*0 == 0 the stations for the second location will start at 100 (e.g. 2.1 CBS becomes 102.1 CBS) and the stations for the third location will start at 200 (e.g. 13.2 WWFF becomes 213.2 WWFF).

Another way to do remapping is to use the `--remap_file=filename` option. You specify a JSON file containing your remappings. To get your current mappings, you can go to `http://PORT:IP/map.json`. Copy that content to a JSON file (you'll want to pretty it up too to make it easier to work with) and you can edit that JSON file, save it, and then use this option to load those remappings the next time you run `locast2tuner`. You will need to restart `locast2tuner` in order to see any changes you made (and you may need to recreate your tuner/EPG setup to have Plex or Emby reflect the right channels). ***NOTE*** This is currently a manual edit process, so if you want to go this route, please be sure that the JSON content is valid JSON before trying to use it. A web-based remap editor is in the works.

## Logging
`locast2tuner` has a few options for logging: directly to the terminal (default), logging to a file and logging through syslog. You can combine the way you want to log by specifying multiple options:

- `--quiet`: disable logging to the terminal
- `--syslog`: log through syslog
- `--logfile <filename>`: log to a file separately

# TODO
This project isn't complete yet. It works, but there are a few things I'd like to get done. These can be found on the [Issues page](https://github.com/wouterdebie/locast2tuner/issues)

# Submitting bugs or feature requests
## Bugs
When you encounter a bug, please use [Github Issues](https://github.com/wouterdebie/locast2tuner/issues):
- _**PLEASE USE THE ISSUE TEMPLATES!**_ Issues that are lacking log excerpts and other information might be closed. In other words don't file issues that are simple "It doesn't work" ones.
- Add a detailed description of the issue you're experiencing.
- Explain what steps can be taken to reproduce the issue.
- If possible, add an excerpt of the log file that shows the error.
- Add a copy of your config. You can get a copy of your running config by opening `/config` in a browser (e.g [http://localhost:6077/config](http://localhost:6077/config)). This will not expose your locast password. If you can't access `/config`, please add your config file *without* your password.
- Before submitting, mark the issue as a "Bug".

## Feature requests
When you have a features you'd like to see added, please use [Github Issues](https://github.com/wouterdebie/locast2tuner/issues) and mark the issue as an "Enhancement".

## Pull requests
If you're so awesome that you want to fix bugs or add features yourself, please fork this repository, code, and create a [Pull Request](https://docs.github.com/en/github/collaborating-with-issues-and-pull-requests/about-pull-requests). Please [squash your commits](https://www.git-tower.com/learn/git/faq/git-squash/) into a single commit before sending the pull request.
