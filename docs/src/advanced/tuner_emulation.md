# Tuner Emulation
`locast2tuner` can act as both a HDHomerun device or as an m3u tuner. Plex mainly supports HDHomerun, while Emby supports both.

In case `locast2tuner` is used as an HDHomerun device it will copy the `mpegts` stream from locast.org to the Media server. This means that the video stream itself will be passed through `locast2tuner`.

When using `locast2tuner` as an m3u tuner, it will pass on the m3u from locast to the media server without any stream interference. This means that the media server will directly connect to the stream.

- For use as a HDHomerun tuner, use `IP:PORT` (defaults to `http://127.0.0.1:6077`) to connect
- For use as an m3u tuner, use `http://IP:PORT/tuner.m3u` (defaults to `http://127.0.0.1:6077/tuner.m3u`) as the URL to connect.
