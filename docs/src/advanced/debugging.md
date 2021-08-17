# Debugging

Media servers communicate with `locast2tuner` using HTTP. The following URLs might come in handy when debugging. By default these URLs are accessible on `http://127.0.0.1:6077`.

|URL|Description|
| - | - |
`/` or `/device.xml` | HDHomerun device.xml
`/discover.json` | HDHomerun discover.json
`/epg.xml` | Electronic Programming Guide in XMLTV format
`/epg` | Electronic Programming Guide in JSON format. This format is mainly used for debugging and is pretty much all the data that was received from locast.org
`/lineup_status.json` | HDHomerun lineup status
`/lineup.json` | HDHomerun lineup.json
`/lineup.post` | URL that HDHomerun uses to trigger a refresh. This doesn't do anything
`/lineup.xml` | HDHomerun lineup.xml
`/map.json` | Shows how [channel mapping](./remapping.md) is currently configured
`/tuner.m3u` | Lineup for m3u tuners
`/watch/{channel_id}.m3u` | Request an m3u stream for a `channel_id`
`/watch/{channel_id}` | Request an mpegts stream for a `channel_id`

You can open the above URLs in a browser and directly see the output.

In order to debug video streams, you can use `ffplay` (part of [`ffmpeg`](https://www.ffmpeg.org/)). E.g. `ffplay http://127.0.0.1/watch/223892.m3u`.
