FROM timberio/vector:0.54.0-debian@sha256:099732c890b095d5222f59bdc82a0579ae3d48b9e2407f3680586dd8d2f75f64

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
