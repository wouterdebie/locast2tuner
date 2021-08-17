# Regular Docker
{{#include ../../locast_account.md}}

In order to use the regular Docker image, a [configuration](../configuration.md) file is required. `locast2tuner` comes with an [example config file](https://raw.githubusercontent.com/wouterdebie/locast2tuner/main/assets/config.example).

```bash
# Create a config directory (e.g. $HOME/.locast2tuner) and copy the example file in there:
mkdir $HOME/.locast2tuner
# Download a sample config file in $HOME/.locast2tuner/config
curl -o $HOME/.locast2tuner/config \
  https://raw.githubusercontent.com/wouterdebie/locast2tuner/main/assets/config.example
# ... edit the file to match your settings ...
```

## Notes
>The Docker image has the path of the configuration file hardcoded to `/app/config/config` inside the container, so make sure the volume mappings are correct.

>`locast2tuner` requires Unix style line endings (i.e. LF) in the config file. This means that in Windows you have to make sure that the configuration file is saved in the right format. Either use an editor that allows you to save with Unix style line endings (like Notepad++ or VS Code) or you can run `dos2unix config` to remove DOS/Windows (i.e. CRLF) line endings prior to launching the container.

## Docker Compose (_recommended_)

If you'd like to use Docker Compose  you can use the sample [docker-compose.yml](https://github.com/wouterdebie/locast2tuner/blob/main/assets/docker/docker-compose.yml) and edit it to match your settings.

In order to run the image using `docker-compose`, make sure you have a valid config file in `$HOME/.locast2tuner/config`:

```sh
# Get the sample docker-compose.yml
curl -o docker-compose.yml \
  https://raw.githubusercontent.com/wouterdebie/locast2tuner/main/assets/docker/docker-compose.yml

# .. edit docker-compose.yml to match your settings ..

# Start the Docker container
docker-compose up -d
```

{{#include upgrading-compose.md}}

## Manual Docker

In order to run the image, make sure you have a valid config file in `$HOME/.locast2tuner/config`.
```sh
# Pull latest locast2tuner image
docker pull ghcr.io/wouterdebie/locast2tuner:latest

# Start the container
docker run \
  -p 6077:6077 \
  -v $HOME/.locast2tuner/:/app/config \
  --name locast2tuner \
  -d ghcr.io/wouterdebie/locast2tuner:latest
```

To upgrade the image:

```sh
# Pull the latest version of the locast2tuner container image
docker pull ghcr.io/wouterdebie/locast2tuner:latest

# Stop the existing container
docker stop locast2tuner

# Remove the existing container
docker rm locast2tuner

# Start the container
docker run \
  -p 6077:6077 \
  -v $HOME/.locast2tuner/:/app/config \
  --name locast2tuner \
  -d ghcr.io/wouterdebie/locast2tuner:latest
```
