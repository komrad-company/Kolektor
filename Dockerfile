FROM timberio/vector:0.44-debian

LABEL maintainer="Benoit Caillabet"
LABEL description="Vector.dev with OCSF catalog — korelator-catalog"

# Copier tout le catalog
COPY catalog/ /etc/vector/catalog/
COPY _lib/ /etc/vector/_lib/
COPY entrypoint.sh /entrypoint.sh

RUN chmod +x /entrypoint.sh

# Variables runtime obligatoires
ENV SOURCE_TYPE=""
ENV TENANT_ID=""
ENV DATASOURCE_ID=""
ENV QUICKWIT_ENDPOINT="http://quickwit-searcher.quickwit:7280"
ENV LISTEN_PORT=""

ENTRYPOINT ["/entrypoint.sh"]
