ARG BUILDPLATFORM
FROM ${BUILDPLATFORM}/alpine:latest
ARG RUSTPLATFORM
VOLUME /srv
EXPOSE 8000/tcp
STOPSIGNAL SIGINT
# Our useful set of tools
RUN apk update && apk upgrade && apk add jq curl netcat-openbsd bash
COPY target/${RUSTPLATFORM}/release/stylus /usr/local/bin/stylus
CMD ["stylus", "/srv/config.yaml"]
