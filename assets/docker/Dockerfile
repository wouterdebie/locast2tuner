FROM alpine:latest

COPY ./amd64binaries/locast2tuner /app
COPY ./assets/docker/run.sh /app

ENTRYPOINT ["bash", "/app/run.sh"]
