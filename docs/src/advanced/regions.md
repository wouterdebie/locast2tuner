# Regions
`locast2tuner` can manipulate regions in various different ways. It can override the location you appear to be coming from and it supports using multiple locations at the same time.

## Location overrides
By default `locast2tuner` uses your IP address to determine your location, but it also allows you to override the [locast.org](https://locast.org) location you're creating a tuner for. This is done by mapping a zip code to geographical coordinates.
Check [locast.org](https://locast.org) for an up-to-date list of available locations.

There are two [configuration](../configuration.md) options available to override the location:

| Option | Description | Examples |
| - | - | - |
|`override_zipcodes` | Use a comma separated list of zip codes | `--override_zipcodes "90210,55111"`(cli)<br>`$l2t_override_zipcodes="90210,55111"` (env vars)<br>`override_zipcodes = ["90210", "55111"]`(config file)
|`override_cities` | Use a pipe separated list of "City, State" | `--override_cities "Los Angeles, CA\|Portland, OR"`(cli)<br>`$l2t_override_cities="Los Angeles, CA\|Portland, OR"` (env vars)<br>`override_cities = ["Los Angeles, CA", "Portland, OR"]`(config file)


A [file with all available locast regions](https://github.com/wouterdebie/locast2tuner/blob/main/assets/regions) is included in the `locast2tuner` distribution.

## Multiple instances
When using multiple locations, `locast2tuner` will start multiple instances. Those instances will be available on TCP ports starting at the value that is specified with the `port` (or the default `6077`) argument and incremented by one. It will also generate UUIDs for each tuner.

>PMS supports multiple devices, but does not support multiple Electronic Programming Guides (EPGs). Emby supports both. I personally use Emby since it allows for multiple EPGs.
