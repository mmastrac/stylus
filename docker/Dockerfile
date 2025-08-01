FROM alpine:latest
ARG BUILDPLATFORM
ARG VERSION
ARG RUSTPLATFORM
ARG PUPVERSION=0.4.0
VOLUME /srv
EXPOSE 8000/tcp

# Our useful set of tools from the Alpine repository
RUN apk update && \
    apk upgrade && \
    apk add \
        # JSON processor for parsing API responses
        jq \
        # YAML processor for parsing YAML configs and responses
        yq \
        # HTTP client for web scraping and API calls
        curl \
        # nc for network connectivity testing
        netcat-openbsd \
        # shell for script execution
        bash \
        # ssh, scp for remote monitoring
        openssh-client \
        # snmpwalk, snmpbulkwalk for SNMP queries
        net-snmp-tools \
        # init system for proper signal handling
        tini \
        # SSL/TLS tools for secure connections
        openssl \
        # SSL wrapper for non-SSL services
        stunnel \
        # ping, ping6, traceroute6, arping
        iputils \
        # nmap, ncat, nping for network scanning
        nmap \
        # dig, nslookup, host for DNS queries
        bind-tools \
        # ps, top, free, vmstat, w for process/system info (container only)
        procps \
        # iostat, mpstat, sar for system statistics (container only)
        sysstat \
        # interactive process viewer (container only)
        htop \
        # text editor for debugging scripts
        vim \
        # pattern matching (explicit)
        grep \
        # stream editor for text manipulation
        sed \
        # text processing and pattern scanning
        gawk \
        # cat, head, tail, sort, uniq, wc, cut, etc.
        coreutils

# Set architecture and install binaries
RUN set -eux; \
    ARCH=$(uname -m); \
    case "$ARCH" in \
      x86_64)   LINUX_ARCH=linux_amd64 ;; \
      aarch64)  LINUX_ARCH=linux_arm64 ;; \
      armv7l|armv6l) LINUX_ARCH=linux_arm ;; \
      i386|i686) LINUX_ARCH=linux_386 ;; \
      *) echo "Unsupported arch: $ARCH" && exit 1 ;; \
    esac; \
    curl -fL "https://github.com/ericchiang/pup/releases/download/v${PUPVERSION}/pup_v${PUPVERSION}_${LINUX_ARCH}.zip" \
      | unzip -p - > /usr/local/bin/pup && chmod a+x /usr/local/bin/pup; \
    curl -fL "https://github.com/mmastrac/stylus/releases/download/${VERSION}/stylus_${LINUX_ARCH}" \
      > /usr/local/bin/stylus && chmod a+x /usr/local/bin/stylus

ENV FORCE_CONTAINER_LISTEN_ADDR=0.0.0.0 FORCE_CONTAINER_PATH=/srv/ FORCE_CONTAINER_PORT=8000
CMD [ "run" ]

# Use tini for proper signal handling
ENV TINI_VERBOSITY=0
ENTRYPOINT [ "tini", "--", "/usr/local/bin/stylus" ]
