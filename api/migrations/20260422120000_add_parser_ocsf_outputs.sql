-- Support parsers that emit several OCSF classes / indexes.

ALTER TABLE kolektor.parsers
    ADD COLUMN IF NOT EXISTS ocsf_outputs JSONB NOT NULL DEFAULT '[]'::jsonb;
