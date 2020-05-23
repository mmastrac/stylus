ARG BUILDPLATFORM
FROM ${BUILDPLATFORM}/alpine:latest
ARG RUSTPLATFORM
VOLUME /srv
EXPOSE 8000/tcp
# Our useful set of tools
RUN apk update && apk upgrade && apk add jq curl netcat-openbsd bash openssh-client net-snmp-tools
COPY target/${RUSTPLATFORM}/release/stylus /usr/local/bin/stylus
CMD ["stylus", "/srv/config.yaml"]
