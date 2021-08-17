# Unraid
{{#include ../locast_account.md}}

Thanks to [RandomNinjaAtk](https://github.com/RandomNinjaAtk) there's a specific x86-64 Docker image for Unraid that is built using this [Dockerfile.unraid](https://github.com/wouterdebie/locast2tuner/blob/main/assets/docker/Dockerfile.unraid). The big difference with the Docker image described above is that this image is configured using environment variables, rather than through an externally mounted configuration file.

The container image is configured with the following parameters passed at runtime:

| Parameter | Function |
| ---- | --- |
| `-p 6077` | The port for the tuner access |
| `-e L2TUSER=username` | Locast Username |
| `-e L2TPASS=password` | Locast Password |
| `-e L2TZIP=#####,#####` | Locast Zipcodes, zipcode in format: #####,#####,##### |
| `-e multiplex=true` | Enables multiplexing |

## Running using docker-compose
Compatible with docker-compose v2 schemas.
```yaml
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

{{#include upgrading-compose.md}}

## Running using docker
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
