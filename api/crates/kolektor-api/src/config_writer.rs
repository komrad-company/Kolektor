use anyhow::{Context, Result};
use kolektor_common::models::Parser;
use std::path::Path;
use tokio::fs;
use toml::{Table, Value};

/// Assemble le contenu TOML Vector agrégé pour tous les parsers actifs.
///
/// Stratégie : Parse chaque parser actif en `toml::Table`, puis fusionne
/// les blocs `sources`, `transforms`, et `sinks`.
/// Substitue `${DATASOURCE_ID}` par `<datasource_base>-<category>-<name>`.
pub fn assemble_toml(parsers: &[Parser], datasource_base: &str) -> String {
    let mut header = String::new();
    header.push_str("# Kolektor — config générée automatiquement, ne pas éditer à la main\n");
    header.push_str(&format!("# Parsers actifs : {}\n\n", parsers.len()));

    if parsers.is_empty() {
        header.push_str(
            "# Aucun parser actif : stub internal_logs -> blackhole pour que Vector démarre.\n\
             [sources._kolektor_idle]\n\
             type = \"internal_logs\"\n\n\
             [sinks._kolektor_blackhole]\n\
             type = \"blackhole\"\n\
             inputs = [\"_kolektor_idle\"]\n\
             print_interval_secs = 0\n",
        );
        return header;
    }

    let mut merged = Table::new();

    for parser in parsers {
        let ds_id = format!(
            "{}-{}",
            datasource_base,
            parser.source_type.replace('/', "-")
        );
        let substituted = parser.vector_toml.replace("${DATASOURCE_ID}", &ds_id);

        let parsed: Table = match toml::from_str(&substituted) {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("Failed to parse TOML for {}: {}", parser.source_type, e);
                continue;
            }
        };

        for (root_key, root_val) in parsed {
            if !merged.contains_key(&root_key) {
                merged.insert(root_key.clone(), Value::Table(Table::new()));
            }

            if let Value::Table(parser_section) = root_val {
                if let Some(Value::Table(global_section)) = merged.get_mut(&root_key) {
                    for (component_name, component_val) in parser_section {
                        if global_section.contains_key(&component_name) {
                            tracing::error!(
                                "Collision detected: component '{}' in section '{}' from parser '{}' already exists",
                                component_name,
                                root_key,
                                parser.source_type
                            );
                        }
                        global_section.insert(component_name, component_val);
                    }
                }
            } else {
                // S'il ne s'agit pas d'une table (cas rare pour vector.toml)
                merged.insert(root_key, root_val);
            }
        }
    }

    let toml_str = toml::to_string(&merged).unwrap_or_else(|e| {
        tracing::error!("Failed to serialize merged TOML: {}", e);
        String::new()
    });

    format!("{}{}", header, toml_str)
}

/// Écrit le fichier cible de façon atomique (tmp + rename).
pub async fn write_atomic(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("creating parent dir {}", parent.display()))?;
    }

    let tmp = path.with_extension("toml.tmp");
    fs::write(&tmp, content.as_bytes())
        .await
        .with_context(|| format!("writing {}", tmp.display()))?;

    fs::rename(&tmp, path)
        .await
        .with_context(|| format!("renaming {} -> {}", tmp.display(), path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn fixture(source_type: &str, vector_toml: &str) -> Parser {
        Parser {
            id: Uuid::now_v7(),
            source_type: source_type.to_string(),
            display_name: source_type.to_string(),
            category: source_type.split('/').next().unwrap_or("").to_string(),
            default_port: None,
            ocsf_class_uid: None,
            ocsf_category_uid: None,
            ocsf_index: None,
            vector_toml: vector_toml.to_string(),
            description: None,
            built_in: true,
            enabled: true,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn empty_parsers_still_produces_header() {
        let out = assemble_toml(&[], "ds-bibihome");
        assert!(out.contains("Parsers actifs : 0"));
        assert!(out.contains("_kolektor_idle"));
        assert!(out.contains("_kolektor_blackhole"));
    }

    #[test]
    fn datasource_id_is_substituted_per_parser() {
        let p1 = fixture(
            "linux/syslog",
            r#"[transforms.x]
source = '.datasource_id = "${DATASOURCE_ID}"'"#,
        );
        let p2 = fixture(
            "network/opnsense",
            r#"[transforms.y]
source = '.datasource_id = "${DATASOURCE_ID}"'"#,
        );
        let out = assemble_toml(&[p1, p2], "ds-bibihome");
        assert!(out.contains("ds-bibihome-linux-syslog"));
        assert!(out.contains("ds-bibihome-network-opnsense"));
        assert!(!out.contains("${DATASOURCE_ID}"));
    }

    #[test]
    fn other_env_vars_are_preserved() {
        let p = fixture(
            "linux/syslog",
            r#"[sources.x]
address = "0.0.0.0:${LISTEN_PORT:-5141}"
tenant = "${TENANT_ID}"
uri = "${QUICKWIT_ENDPOINT}/api/v1/x/ingest""#,
        );
        let out = assemble_toml(&[p], "ds-bibihome");
        assert!(out.contains("${LISTEN_PORT:-5141}"));
        assert!(out.contains("${TENANT_ID}"));
        assert!(out.contains("${QUICKWIT_ENDPOINT}"));
    }

    #[tokio::test]
    async fn write_atomic_creates_parent() {
        let tmp = tempdir_path();
        let target = tmp.join("sub/dir/sources.toml");
        write_atomic(&target, "hello").await.unwrap();
        let got = tokio::fs::read_to_string(&target).await.unwrap();
        assert_eq!(got, "hello");
    }

    #[tokio::test]
    async fn write_atomic_overwrites_existing() {
        let tmp = tempdir_path();
        let target = tmp.join("sources.toml");
        write_atomic(&target, "v1").await.unwrap();
        write_atomic(&target, "v2").await.unwrap();
        let got = tokio::fs::read_to_string(&target).await.unwrap();
        assert_eq!(got, "v2");
    }

    fn tempdir_path() -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("kolektor-test-{}", Uuid::now_v7()));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}
