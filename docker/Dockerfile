ARG BUILDPLATFORM
FROM ${BUILDPLATFORM}/alpine:latest
ARG RUSTPLATFORM
VOLUME /srv
EXPOSE 8000/tcp
# Our useful set of tools
RUN apk update && apk upgrade && apk add jq curl netcat-openbsd bash openssh-client net-snmp-tools tini
COPY target/${RUSTPLATFORM}/release/stylus /usr/local/bin/stylus
ENV FORCE_CONTAINER_LISTEN_ADDR=0.0.0.0 FORCE_CONTAINER_PATH=/srv/config.yaml FORCE_CONTAINER_PORT=8000
CMD []

# Use tini for proper signal handling
ENV TINI_VERBOSITY=0
ENTRYPOINT [ "tini", "--", "/usr/local/bin/stylus" ]
