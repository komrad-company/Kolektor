FROM timberio/vector:0.55.0-debian@sha256:a4be1111b40303524aae2ffb02cd59cef2a4e9753bd13d265bf0233e921828d9

# OS CVE patching — see .hadolint.yaml for the DL3005 justification.
# curl is required by /init/create-indexes.sh (Quickwit index bootstrap).
RUN apt-get update \
 && apt-get upgrade -y \
 && apt-get install -y --no-install-recommends curl ca-certificates \
 && rm -rf /var/lib/apt/lists/*

LABEL maintainer="Benoit Caillabet"
LABEL description="Vector.dev + Kolektor parser catalog"

# Vector parses hostile raw logs: run it unprivileged. All listeners bind
# ports above 1024 and /var/lib/vector holds the file-source checkpoints.
RUN useradd --system --no-create-home --shell /usr/sbin/nologin vector \
 && mkdir -p /var/lib/vector \
 && chown vector /var/lib/vector

COPY catalog/ /etc/vector/catalog/
COPY init/indexes/ /init/indexes/
COPY --chmod=0755 init/create-indexes.sh /init/create-indexes.sh
COPY --chmod=0755 entrypoint.sh /entrypoint.sh

USER vector

ENTRYPOINT ["/entrypoint.sh"]
