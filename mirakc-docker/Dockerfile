# This file works properly only with Docker.
# Assumed that docker.io registry is used.
#
# `podman buildx build` doesn't work at this point because it doesn't support
# the following build arguments that docker set automagically:
#
#   BUILDPLATFORM
#   TARGETPLATFORM
#   TARGETOS
#   TARGETARCH
#   TARGETVARIANT

ARG TOOLS_TAG=debian
# We set `sid` here in order to avoid the `InvalidDefaultArgInFrom` warning,
# but this argument is always specified in build commands.  As a result, `sid`
# is never used.
ARG DEBIAN_CODENAME=sid

FROM mirakc/tools:$TOOLS_TAG AS mirakc-tools

FROM --platform=$BUILDPLATFORM mirakc/buildenv:debian-$TARGETOS-$TARGETARCH$TARGETVARIANT AS mirakc-build
# COPY . ./
RUN git clone --recursive https://github.com/mirakc/mirakc.git ./
ARG BUILDPLATFORM
ARG TARGETPLATFORM
ARG PROFILE=release
RUN sh ./docker/build-scripts/mirakc.sh debian $BUILDPLATFORM $TARGETPLATFORM $PROFILE

FROM debian:$DEBIAN_CODENAME-slim AS mirakc
LABEL maintainer="Contributors of mirakc"
COPY --from=mirakc-tools /usr/local/bin /usr/local/bin/
COPY --from=mirakc-build /usr/local/bin/mirakc /usr/local/bin/
COPY --from=mirakc-build /build/resources/strings.yml /etc/mirakc/strings.yml
ARG PROFILE=release

RUN 
RUN set -eux \
 export DEBIAN_FRONTEND=noninteractive \
 && apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl dvb-tools jq socat libpcsclite1 \
 && curl -sSL -o recisdb_1.2.3-1_amd64.deb https://github.com/kazuki0824/recisdb-rs/releases/download/1.2.3/recisdb_1.2.3-1_amd64.deb \
 && dpkg -i ./recisdb_1.2.3-1_amd64.deb \
 && if [ "$PROFILE" = perf ]; then apt-get install -y --no-install-recommends heaptrack valgrind; fi \
 # cleanup
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/* \
 && rm -rf /var/tmp/* \
 && rm -rf /tmp/* \
 && rm -f ./recisdb_1.2.3-1_amd64.deb
 RUN curl -sSL -o /usr/local/bin/isdb-scanner https://github.com/tsukumijima/ISDBScanner/releases/download/v1.3.1/isdb-scanner && chmod +x /usr/local/bin/isdb-scanner

 ENV MIRAKC_CONFIG=/etc/mirakc/config.yml
ENV MIRAKC_BUILD_PROFILE=$PROFILE

COPY ./mirakc-init.sh ./

EXPOSE 40772
ENTRYPOINT ./mirakc-init.sh

# ENTRYPOINT ["/usr/local/bin/mirakc"]
CMD []

# FROM debian:$DEBIAN_CODENAME-slim AS timeshift-fs
# LABEL maintainer="Contributors of mirakc"
# COPY --from=mirakc-build /usr/local/bin/mirakc-timeshift-fs /usr/local/bin/
# COPY --from=mirakc-build /usr/local/bin/run-mirakc-timeshift-fs /usr/local/bin/
# COPY --from=mirakc-build /build/resources/strings.yml /etc/mirakc/strings.yml
# ARG PROFILE=release
# RUN set -eux \
#  && export DEBIAN_FRONTEND=noninteractive \
#  && apt-get update \
#  && apt-get install -y --no-install-recommends fuse3 \
#  && echo 'user_allow_other' >/etc/fuse.conf \
#  && if [ "$PROFILE" = perf ]; then apt-get install -y --no-install-recommends heaptrack valgrind; fi \
#  # cleanup
#  && apt-get clean \
#  && rm -rf /var/lib/apt/lists/* \
#  && rm -rf /var/tmp/* \
#  && rm -rf /tmp/*
# ENV MIRAKC_CONFIG=/etc/mirakc/config.yml
# ENV MIRAKC_BUILD_PROFILE=$PROFILE
# ENTRYPOINT ["/usr/local/bin/run-mirakc-timeshift-fs"]
