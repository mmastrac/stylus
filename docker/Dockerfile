ARG BUILDPLATFORM
FROM ${BUILDPLATFORM}/alpine:latest
ARG VERSION
ARG RUSTPLATFORM
ARG BINARYPLATFORM
ARG PUPVERSION=0.4.0
VOLUME /srv
EXPOSE 8000/tcp

# Our useful set of tools from the Alpine repository
RUN apk update && \
    apk upgrade && \
    apk add jq curl netcat-openbsd bash openssh-client net-snmp-tools tini openssl stunnel

# Install pup via curl
RUN curl -fL https://github.com/ericchiang/pup/releases/download/v${PUPVERSION}/pup_v${PUPVERSION}_${BINARYPLATFORM}.zip \
    | unzip -p - > /usr/local/bin/pup \
    && chmod a+x /usr/local/bin/pup

# Install stylus via curl
RUN curl -fL https://github.com/mmastrac/stylus/releases/download/${VERSION}/stylus_${BINARYPLATFORM} > /usr/local/bin/stylus \
    && chmod a+x /usr/local/bin/stylus

ENV FORCE_CONTAINER_LISTEN_ADDR=0.0.0.0 FORCE_CONTAINER_PATH=/srv/config.yaml FORCE_CONTAINER_PORT=8000
CMD []

# Use tini for proper signal handling
ENV TINI_VERBOSITY=0
ENTRYPOINT [ "tini", "--", "/usr/local/bin/stylus" ]
