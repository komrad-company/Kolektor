use anyhow::{Context, Result};
use serde::Deserialize;
use sqlx::PgPool;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub display_name: String,
    pub default_port: Option<i32>,
    pub ocsf_class_uid: Option<i32>,
    pub ocsf_category_uid: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct CatalogParser {
    pub source_type: String,
    pub display_name: String,
    pub category: String,
    pub default_port: Option<i32>,
    pub ocsf_class_uid: Option<i32>,
    pub ocsf_category_uid: Option<i32>,
    pub ocsf_index: Option<String>,
    pub vector_toml: String,
    pub description: Option<String>,
}

/// Scanne le répertoire catalog et retourne la liste des parsers détectés.
pub fn scan_catalog(catalog_dir: &Path) -> Result<Vec<CatalogParser>> {
    let mut parsers = Vec::new();

    for category_entry in std::fs::read_dir(catalog_dir)
        .with_context(|| format!("reading catalog dir {}", catalog_dir.display()))?
    {
        let category_entry = category_entry?;
        if !category_entry.file_type()?.is_dir() {
            continue;
        }
        let category = category_entry.file_name().to_string_lossy().to_string();
        if category.starts_with('_') || category.starts_with('.') {
            continue;
        }

        for parser_entry in std::fs::read_dir(category_entry.path())? {
            let parser_entry = parser_entry?;
            if !parser_entry.file_type()?.is_dir() {
                continue;
            }
            let name = parser_entry.file_name().to_string_lossy().to_string();
            let vector_toml_path = parser_entry.path().join("vector.toml");
            let manifest_path = parser_entry.path().join("manifest.yaml");
            if !vector_toml_path.exists() || !manifest_path.exists() {
                if vector_toml_path.exists() && !manifest_path.exists() {
                    tracing::warn!("Skipping {} because manifest.yaml is missing", name);
                }
                continue;
            }

            match parse_parser_dir(&category, &name, &parser_entry.path(), &vector_toml_path, &manifest_path) {
                Ok(parser) => parsers.push(parser),
                Err(e) => tracing::warn!("Failed to parse {}: {}", name, e),
            }
        }
    }

    parsers.sort_by(|a, b| a.source_type.cmp(&b.source_type));
    Ok(parsers)
}

fn parse_parser_dir(
    category: &str,
    name: &str,
    parser_dir: &Path,
    vector_toml_path: &Path,
    manifest_path: &Path,
) -> Result<CatalogParser> {
    let vector_toml = std::fs::read_to_string(vector_toml_path)
        .with_context(|| format!("reading {}", vector_toml_path.display()))?;

    let manifest_content = std::fs::read_to_string(manifest_path)
        .with_context(|| format!("reading manifest {}", manifest_path.display()))?;
    
    let manifest: Manifest = serde_yaml::from_str(&manifest_content)
        .with_context(|| format!("parsing yaml {}", manifest_path.display()))?;

    let source_type = format!("{category}/{name}");
    let ocsf_index = ocsf_index_for(manifest.ocsf_class_uid);
    let description = read_description(parser_dir);

    Ok(CatalogParser {
        source_type,
        display_name: manifest.display_name,
        category: category.to_string(),
        default_port: manifest.default_port,
        ocsf_class_uid: manifest.ocsf_class_uid,
        ocsf_category_uid: manifest.ocsf_category_uid,
        ocsf_index,
        vector_toml,
        description,
    })
}

fn ocsf_index_for(class_uid: Option<i32>) -> Option<String> {
    match class_uid? {
        4001 => Some("ocsf-network".into()),
        6001 => Some("ocsf-audit".into()),
        3001 | 3002 => Some("ocsf-identity".into()),
        1001 | 1003 | 2001 => Some("ocsf-endpoint".into()),
        _ => None,
    }
}

fn read_description(parser_dir: &Path) -> Option<String> {
    let readme: PathBuf = parser_dir.join("README.md");
    let content = std::fs::read_to_string(readme).ok()?;
    let first_para: String = content
        .lines()
        .skip_while(|l| l.trim().is_empty() || l.starts_with('#'))
        .take_while(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let trimmed = first_para.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Insère / met à jour les parsers en DB. Préserve `enabled`.
/// Incrémente `version` uniquement si `vector_toml` change.
pub async fn seed(pool: &PgPool, catalog_dir: &Path) -> Result<SeedReport> {
    let parsers = scan_catalog(catalog_dir)?;
    let mut report = SeedReport::default();

    for parser in parsers {
        let id = Uuid::now_v7();
        let inserted: bool = sqlx::query_scalar(
            r#"
            INSERT INTO kolektor.parsers (
                id, source_type, display_name, category, default_port,
                ocsf_class_uid, ocsf_category_uid, ocsf_index,
                vector_toml, description, built_in, enabled, version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10, true, false, 1)
            ON CONFLICT (source_type) DO UPDATE SET
                display_name      = EXCLUDED.display_name,
                category          = EXCLUDED.category,
                default_port      = EXCLUDED.default_port,
                ocsf_class_uid    = EXCLUDED.ocsf_class_uid,
                ocsf_category_uid = EXCLUDED.ocsf_category_uid,
                ocsf_index        = EXCLUDED.ocsf_index,
                description       = EXCLUDED.description,
                vector_toml       = EXCLUDED.vector_toml,
                version           = kolektor.parsers.version
                                      + CASE WHEN kolektor.parsers.vector_toml <> EXCLUDED.vector_toml
                                             THEN 1 ELSE 0 END,
                updated_at        = CASE WHEN kolektor.parsers.vector_toml <> EXCLUDED.vector_toml
                                         THEN now() ELSE kolektor.parsers.updated_at END
            RETURNING (xmax = 0)
            "#,
        )
        .bind(id)
        .bind(&parser.source_type)
        .bind(&parser.display_name)
        .bind(&parser.category)
        .bind(parser.default_port)
        .bind(parser.ocsf_class_uid)
        .bind(parser.ocsf_category_uid)
        .bind(&parser.ocsf_index)
        .bind(&parser.vector_toml)
        .bind(&parser.description)
        .fetch_one(pool)
        .await
        .with_context(|| format!("upserting parser {}", parser.source_type))?;

        if inserted {
            report.inserted += 1;
        } else {
            report.updated += 1;
        }
    }

    Ok(report)
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SeedReport {
    pub inserted: usize,
    pub updated: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_mapping() {
        assert_eq!(ocsf_index_for(Some(4001)), Some("ocsf-network".into()));
        assert_eq!(ocsf_index_for(Some(6001)), Some("ocsf-audit".into()));
        assert_eq!(ocsf_index_for(Some(3001)), Some("ocsf-identity".into()));
        assert_eq!(ocsf_index_for(Some(1003)), Some("ocsf-endpoint".into()));
        assert_eq!(ocsf_index_for(None), None);
    }
}
