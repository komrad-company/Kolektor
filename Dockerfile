FROM timberio/vector:0.54.0-debian

LABEL maintainer="Benoit Caillabet"
LABEL description="Vector.dev with OCSF catalog — kolektor"

# Copier tout le catalog
COPY catalog/ /etc/vector/catalog/
COPY entrypoint.sh /entrypoint.sh

RUN chmod +x /entrypoint.sh

# Variables runtime : entrypoint.sh valide leur presence au demarrage.
# Pas de default pour QUICKWIT_ENDPOINT : chaque deploiement doit fournir explicitement
# son URL (http en cluster interne, https en prod) pour eviter un fallback silencieux.

ENTRYPOINT ["/entrypoint.sh"]
