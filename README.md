# locast2tuner

**DEPRECATION WARNING! Unfortunately locast.org is no longer. This means that this project has been deprecated..**

[![Join the chat at https://gitter.im/wouterdebie/locast2tuner](https://badges.gitter.im/wouterdebie/locast2tuner.svg)](https://gitter.im/wouterdebie/locast2tuner?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge) [![GitHub release](https://img.shields.io/github/v/release/wouterdebie/locast2tuner)](https://github.com/wouterdebie/locast2tuner/releases/latest) ![License](https://img.shields.io/badge/License-MIT-blue) [![Sponsor](https://img.shields.io/github/sponsors/wouterdebie)](https://github.com/sponsors/wouterdebie)

This application provides an interface between locast.org and Media Servers like [Plex Media Server (PMS)](https://plex.tv) and [Emby](https://emby.media) by acting like an [HDHomerun](https://www.silicondust.com/) or an m3u tuner and an XMLTV provider.

`locast2tuner` can imitate one or more digital tuners and provides geo cloaking across regions.

> ❗ Since locast.org uses Amazon Cloudfront to stream, region restrictions imposed by Cloudfront can not be circumvented. This means that `locast2tuner` might not work outside of the United States.
# Features
- Override your location using [zip codes or cities](https://wouterdebie.github.io/locast2tuner/advanced/regions.html)
- Multiple digital tuners in a single server, either as separate servers or as one ([multiplexing](https://wouterdebie.github.io/locast2tuner/advanced/multiplexing.html))
- Acts like either an [HDHomerun or an m3u tuner](https://wouterdebie.github.io/locast2tuner/advanced/tuner_emulation.html)
- Provides locast.org [EPG](https://wouterdebie.github.io/locast2tuner/advanced/epg.html) information as an XMLTV guide
- And [many more](https://wouterdebie.github.io/locast2tuner/)

# Documentation
Documentation can be found at [https://wouterdebie.github.io/locast2tuner/](https://wouterdebie.github.io/locast2tuner)

# Sponsoring
If you use `locast2tuner` and want to contribute by sponsoring, please leave a donation at my [Sponsor Page](https://github.com/sponsors/wouterdebie). All of the money donated will go towards organizations that support women and underprivileged children in STEM education like [BUILT BY GIRLS](https://www.builtbygirls.com/), [Electric Girls](https://www.electricgirls.org/), [Project Exploration](https://projectexploration.org/) and others.

# Todo
This project isn't complete yet. It works, but there are a few things I'd like to get done. These can be found on the [Issues page](https://github.com/wouterdebie/locast2tuner/issues)

# Submitting bugs or feature requests
## Bugs
> ❗ Make sure to check https://github.com/wouterdebie/locast2tuner/releases for breaking configuration changes in the latest release before submitting a bug! Also, please read the documentation and FAQ before submitting a bug request!

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
