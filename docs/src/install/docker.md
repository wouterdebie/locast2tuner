# Docker
{{#include ../locast_account.md}}

A Docker image is available from `ghcr.io/wouterdebie/locast2tuner:latest` and is built from this [Dockerfile](https://github.com/wouterdebie/locast2tuner/blob/main/assets/docker/Dockerfile).

The container image is configured so that all [configuration](../../configuration.md) parameters can be set as environment variables set at runtime.

> `l2t_username` and `l2t_password` are required parameters. The rest are optional.

## Run using docker-compose (recommended)
If you'd like to use Docker Compose you can use the sample [docker-compose.yml](https://github.com/wouterdebie/locast2tuner/blob/main/assets/docker/docker-compose.yml) and edit it to match your settings:


```yaml
{{#include ../../../assets/docker/docker-compose.yml}}
```

To configure and run using `docker-compose`:
```sh
# Get the sample docker-compose.yml
curl -o docker-compose.yml \
  https://raw.githubusercontent.com/wouterdebie/locast2tuner/main/assets/docker/docker-compose.yml

# .. edit docker-compose.yml to match your settings ..

# Start the Docker container
docker-compose up -d
```

> The path `/app/config` is available in the container and can be mounted from the host in order to use settings like `l2t_remap_file` or even `l2t_config`.

### Upgrading

```sh
# Pull any new versions of images referenced in docker-compose.yml
docker-compose pull

# Restart any containers in `docker-compose.yml` with newly pulled images
docker-compose up -d
```


## Run using Docker
```sh
# Pull latest locast2tuner image
docker pull ghcr.io/wouterdebie/locast2tuner:latest

docker create \
  --name=locast2tuner \
  -p 6077 \
  -e l2t_username=username \
  -e l2t_password=password \
  -e l2t_override_zipcodes=#####,##### \
  -e l2t_multiplex=true \
  -e l2t_remap=true \
  ghcr.io/wouterdebie/locast2tuner:latest
```

### Upgrading
To upgrade the image:

```sh
# Pull the latest version of the locast2tuner container image
docker pull ghcr.io/wouterdebie/locast2tuner:latest

# Stop the existing container
docker stop locast2tuner

# Remove the existing container
docker rm locast2tuner
```


