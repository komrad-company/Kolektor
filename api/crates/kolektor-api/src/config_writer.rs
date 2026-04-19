use anyhow::{Context, Result};
use kolektor_common::models::Parser;
use std::path::Path;
use tokio::fs;

/// Assemble le contenu TOML Vector agrégé pour tous les parsers actifs.
///
/// Stratégie : concaténation des vector_toml bruts, avec substitution de
/// `${DATASOURCE_ID}` par `<datasource_base>-<category>-<name>` pour que chaque
/// parser ait un datasource_id distinct (même logique que l'ancien entrypoint.sh).
/// Les autres variables (`${TENANT_ID}`, `${QUICKWIT_ENDPOINT}`, `${LISTEN_PORT}`)
/// sont laissées intactes pour que Vector les expande au runtime.
pub fn assemble_toml(parsers: &[Parser], datasource_base: &str) -> String {
    let mut out = String::new();
    out.push_str("# Kolektor — config générée automatiquement, ne pas éditer à la main\n");
    out.push_str(&format!("# Parsers actifs : {}\n\n", parsers.len()));

    if parsers.is_empty() {
        out.push_str(
            "# Aucun parser actif : stub internal_logs -> blackhole pour que Vector démarre.\n\
             [sources._kolektor_idle]\n\
             type = \"internal_logs\"\n\n\
             [sinks._kolektor_blackhole]\n\
             type = \"blackhole\"\n\
             inputs = [\"_kolektor_idle\"]\n\
             print_interval_secs = 0\n",
        );
        return out;
    }

    for parser in parsers {
        let ds_id = format!(
            "{}-{}",
            datasource_base,
            parser.source_type.replace('/', "-")
        );
        let substituted = parser.vector_toml.replace("${DATASOURCE_ID}", &ds_id);

        out.push_str(&format!(
            "# ============================================================\n\
             # {} (source_type={}, version={})\n\
             # datasource_id = {}\n\
             # ============================================================\n",
            parser.display_name, parser.source_type, parser.version, ds_id
        ));
        out.push_str(&substituted);
        if !substituted.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }

    out
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
        let out = assemble_toml(&[], "ds-acme");
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
        let out = assemble_toml(&[p1, p2], "ds-acme");
        assert!(out.contains(r#".datasource_id = "ds-acme-linux-syslog""#));
        assert!(out.contains(r#".datasource_id = "ds-acme-network-opnsense""#));
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
        let out = assemble_toml(&[p], "ds-acme");
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
