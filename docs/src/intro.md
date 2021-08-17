# Introduction

locast2tuner provides an interface between locast.org and [Plex Media Server (PMS)](https://plex.tv) or [Emby](https://emby.media) by acting like an [HDHomerun](https://www.silicondust.com/) or an m3u tuner and an XMLTV provider.

`locast2tuner` can imitate one or more digital tuners and provides geo cloaking across regions.
## Features
- Override your location using ZIP code or GPS coordinates
- Multiple digital tuners in a single server, either as separate servers or as one (multiplexing)
- Acts like either an HDHomerun or an m3u tuner
- Provides locast EPG information as an XMLTV guide

## Background
`locast2tuner` is a rewrite in Rust of [locast2dvr](https://github.com/wouterdebie/locast2dvr), which in turn is a rewrite of [locast2plex](https://github.com/tgorgdotcom/locast2plex). Thanks to the locast2plex developers for writing it and figuring out how to stitch things together!

I rewrote locast2plex to be able to more easily add functionality, use libraries wherever possible (like HTTP, m3u, starting multiple devices, etc), heavily document, generally write clean code, and provide a better user experience (command line argument parsing, automatic download of FCC facilities, etc). And since python's GIL gave me a headache, I rewrote the whole thing in Rust.

Apart from the fact that everything is Rust now, the big difference between `locast2tuner` and `locast2dvr` is that `locast2tuner` does not require ffmpeg anymore. Actually, I completely dropped support for it and only implemented the `direct mode` that `locast2dvr` supports. Next to that, I removed a few debugging features (like `--multiplex-debug`), that don't seem to be used.

Even though this project started as a locast to PMS interface, it's more focused on integrating locast with Emby, since Emby provides a bit more functionality when it comes to Live TV and Tuner (like m3u tuners, XMLTV, etc).
