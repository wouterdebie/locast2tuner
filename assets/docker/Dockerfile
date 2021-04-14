FROM alpine:latest

COPY ./amd64binaries/locast2tuner /
ENTRYPOINT ["/locast2tuner", "--config", "/app/config/config", "-b", "0.0.0.0", "--force_timestamps"]
