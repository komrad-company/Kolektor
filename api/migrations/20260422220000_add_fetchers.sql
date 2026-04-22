-- Fetcher configs are controlled by the UI/API and consumed by kolektor-fetcher.
-- Secrets should normally be referenced by environment variable names in config,
-- not stored directly in this JSONB document.

CREATE TABLE IF NOT EXISTS kolektor.fetchers (
    id                  UUID PRIMARY KEY,
    name                TEXT NOT NULL,
    provider            TEXT NOT NULL,
    parser_source_type  TEXT NOT NULL,
    enabled             BOOLEAN NOT NULL DEFAULT false,
    interval_seconds    INT NOT NULL DEFAULT 300,
    output_path         TEXT NOT NULL,
    config              JSONB NOT NULL DEFAULT '{}'::jsonb,
    state               JSONB NOT NULL DEFAULT '{}'::jsonb,
    last_attempt_at     TIMESTAMPTZ,
    last_success_at     TIMESTAMPTZ,
    last_error          TEXT,
    version             INT NOT NULL DEFAULT 1,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fetchers_provider_chk CHECK (
        provider IN ('microsoft_graph', 'microsoft365_management', 's3')
    ),
    CONSTRAINT fetchers_interval_chk CHECK (interval_seconds >= 30),
    CONSTRAINT fetchers_parser_source_type_fk FOREIGN KEY (parser_source_type)
        REFERENCES kolektor.parsers(source_type) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS fetchers_enabled_idx ON kolektor.fetchers (enabled);
CREATE INDEX IF NOT EXISTS fetchers_provider_idx ON kolektor.fetchers (provider);
CREATE INDEX IF NOT EXISTS fetchers_parser_source_type_idx ON kolektor.fetchers (parser_source_type);
