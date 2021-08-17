### Upgrading

```sh
# Pull any new versions of images referenced in docker-compose.yml
docker-compose pull
# Restart any containers in `docker-compose.yml` with newly pulled images
docker-compose up -d
```
