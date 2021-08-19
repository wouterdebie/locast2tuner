# Unraid
{{#include ../../locast_account.md}}

Thanks to [RandomNinjaAtk](https://github.com/RandomNinjaAtk) there's a specific x86-64 Docker image for Unraid that is built using this [Dockerfile.unraid](https://github.com/wouterdebie/locast2tuner/blob/main/assets/docker/Dockerfile.unraid). The big difference with the Docker image described above is that this image is configured using environment variables, rather than through an externally mounted configuration file.

The container image is configured with all [configuration](../../configuration.md) parameters as environment variables passed at runtime.

> `l2t_username` and `l2t_password` are required parameters. The rest is optional.

## Running using docker-compose
Compatible with docker-compose v2 schemas.
```yaml
version: "2.1"
services:
  locast2tuner:
    image: ghcr.io/wouterdebie/locast2tuner-unraid:latest
    container_name: locast2tuner
    environment:
      - l2t_username=username
      - l2t_password=password
      - l2t_override_zipcodes=#####,#####
      - l2t_multiplex=true
      - l2t_remap=true
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
  -e l2t_username=username \
  -e l2t_password=password \
  -e l2t_override_zipcodes=#####,##### \
  -e l2t_multiplex=true \
  -e l2t_remap=true \
  ghcr.io/wouterdebie/locast2tuner-unraid:latest
```
