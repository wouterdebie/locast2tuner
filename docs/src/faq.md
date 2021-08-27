# Frequently Asked Questions

## I'm getting a "Login failed". What's up?
A few things could be up. First, make sure you can login to locast.org in a browser and make sure you have an active donation. If you haven't already, try running `locast2tuner` from the command line with a higher `verbose` level like:

```
locast2tuner -v3 --config /your/config --other_options_you_have_specified
```

It should show you why you couldn't log in. A 404 error means that your login credentials are unknown. Sometimes locast.org returns a HTTP 204, which means that login succeeded, but that no data was returned. This might happen sporadically and it seems to be an intermittent failure. Or it could have something to do with your network connection or a proxy that doesn't do what you want it to do.


