# Docker
{{#include ../locast_account.md}}

There are two docker images available:

- A "regular" image that is available from `ghcr.io/wouterdebie/locast2tuner:latest` and is built from this [Dockerfile](https://github.com/wouterdebie/locast2tuner/blob/main/assets/docker/Dockerfile).
- An [Unraid](https://unraid.net) image that is available from `ghcr.io/wouterdebie/locast2tuner-unraid:latest` and is built from this [Dockerfile](https://github.com/wouterdebie/locast2tuner/blob/main/assets/docker/Dockerfile.unraid).

The main difference between the images is that the regular image allows you to use a config file and thus configure locast2tuner to your hearts content, while the Unraid image is configured through only a few environment variables with some sane defaults. The Unraid image is (obviously) recommended when running on an Unraid installation.

