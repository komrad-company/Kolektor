FROM timberio/vector:0.54.0-debian

# Patch des CVEs OS — voir .hadolint.yaml pour la justification de DL3005.
RUN apt-get update \
 && apt-get upgrade -y \
 && rm -rf /var/lib/apt/lists/*

LABEL maintainer="Benoit Caillabet"
LABEL description="Vector.dev + Kolektor parser catalog"

COPY catalog/ /etc/vector/catalog/
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
