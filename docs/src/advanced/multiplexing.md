# Multiplexing
`locast2tuner` normally starts an HTTP instance for each Tuner, starting at `port` (default `6077`). But with the option `multiplex`, it will start a single HTTP interface multiplexing all Tuners through one interface for both streaming and EPG.

For example: if you use `--multiplex --override_zipcodes=90210,55111`, all channels from both ZIP codes will be available, but multiplexed at `http://localhost:6077`.

>This type of multiplexing makes sense in Emby, since you can add a single tuner at `http://PORT:IP` or `http://PORT:IP/tuner.m3u` and a single EPG at `http://PORT:IP/epg.xml`
