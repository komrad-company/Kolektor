-- Kolektor API — schéma initial
-- Schema isolé pour cohabiter avec Kontrol sur la même instance PG

CREATE SCHEMA IF NOT EXISTS kolektor;

CREATE TABLE IF NOT EXISTS kolektor.parsers (
    id                 UUID PRIMARY KEY,
    source_type        TEXT NOT NULL UNIQUE,
    display_name       TEXT NOT NULL,
    category           TEXT NOT NULL,
    default_port       INT,
    ocsf_class_uid     INT,
    ocsf_category_uid  INT,
    ocsf_index         TEXT,
    vector_toml        TEXT NOT NULL,
    description        TEXT,
    built_in           BOOLEAN NOT NULL DEFAULT true,
    enabled            BOOLEAN NOT NULL DEFAULT false,
    version            INT NOT NULL DEFAULT 1,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS parsers_enabled_idx ON kolektor.parsers (enabled);
CREATE INDEX IF NOT EXISTS parsers_category_idx ON kolektor.parsers (category);

CREATE TABLE IF NOT EXISTS kolektor.api_tokens (
    id            UUID PRIMARY KEY,
    name          TEXT NOT NULL,
    token_hash    TEXT NOT NULL,
    tenant_id     TEXT NOT NULL,
    last_used_at  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS api_tokens_tenant_idx ON kolektor.api_tokens (tenant_id);

CREATE TABLE IF NOT EXISTS kolektor.sync_events (
    id          UUID PRIMARY KEY,
    event_type  TEXT NOT NULL,
    parser_id   UUID REFERENCES kolektor.parsers(id) ON DELETE SET NULL,
    payload     JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS sync_events_created_idx ON kolektor.sync_events (created_at DESC);
