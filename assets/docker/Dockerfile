FROM alpine:latest

RUN mkdir -p /app/config

COPY ./amd64binaries/locast2tuner /app/
COPY ./assets/docker/run.sh /app/
RUN apk add --no-cache bash

ENTRYPOINT ["/bin/bash", "/app/run.sh"]
