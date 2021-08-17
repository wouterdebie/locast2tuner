# Regions
`locast2tuner` can manipulate regions in various different ways. It can override the location you appear to be coming from and it supports using multiple locations at the same time.

## Location overrides
By default `locast2tuner` uses your IP address to determine your location, but it also allows you to override the [locast.org](https://locast.org) location you're creating a tuner for. This is done by mapping a zip code to geographical coordinates.

As a command line argument, `override_zipcodes` takes a comma separated list of ZIP codes as an argument. E.g. `--override_zipcodes 90210,55111` for Los Angeles and Minneapolis.

Or you can use `override_zipcodes` in a configuration file as an array of strings. E.g. `override_zipcodes = ["90210", "55111"]`.

A [file with all available locast regions](https://github.com/wouterdebie/locast2tuner/blob/main/assets/regions) is included in the `locast2tuner` distribution.

## Multiple instances
When using multiple locations, `locast2tuner` will start multiple instances. Those instances will be available on TCP ports starting at the value that is specified with the `port` (or the default `6077`) argument and incremented by one. It will also generate UUIDs for each tuner.

>PMS supports multiple devices, but does not support multiple Electronic Programming Guides (EPGs). Emby supports both. I personally use Emby since it allows for multiple EPGs.
