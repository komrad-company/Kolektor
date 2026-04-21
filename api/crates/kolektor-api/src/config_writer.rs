use anyhow::{Context, Result, bail};
use kolektor_common::models::Parser;
use std::collections::BTreeMap;
use std::path::Path;
use tokio::fs;
use toml::{Table, Value};
use uuid::Uuid;

/// Assemble le contenu TOML Vector agrégé pour tous les parsers actifs.
///
/// Stratégie : Parse chaque parser actif en `toml::Table`, puis fusionne
/// les blocs `sources`, `transforms`, et `sinks`.
/// Substitue `${DATASOURCE_ID}` par `<datasource_base>-<category>-<name>`.
pub fn assemble_toml(parsers: &[Parser], datasource_base: &str) -> Result<String> {
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
        return Ok(header);
    }

    let mut merged = Table::new();
    let mut source_addresses = BTreeMap::<String, String>::new();

    for parser in parsers {
        let ds_id = format!(
            "{}-{}",
            datasource_base,
            parser.source_type.replace('/', "-")
        );
        let substituted = substitute_runtime_vars(parser, &ds_id);

        let parsed: Table = toml::from_str(&substituted)
            .with_context(|| format!("parsing TOML for parser {}", parser.source_type))?;

        for (root_key, root_val) in parsed {
            if !merged.contains_key(&root_key) {
                merged.insert(root_key.clone(), Value::Table(Table::new()));
            }

            if let Value::Table(parser_section) = root_val {
                if let Some(Value::Table(global_section)) = merged.get_mut(&root_key) {
                    for (component_name, component_val) in parser_section {
                        if global_section.contains_key(&component_name) {
                            bail!(
                                "component collision: [{}.{component_name}] from parser {} already exists",
                                root_key,
                                parser.source_type
                            );
                        }
                        if root_key == "sources" {
                            detect_address_collision(
                                &mut source_addresses,
                                &component_name,
                                &component_val,
                                &parser.source_type,
                            )?;
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

    let toml_str = toml::to_string(&merged).context("serializing merged TOML")?;

    Ok(format!("{}{}", header, toml_str))
}

fn substitute_runtime_vars(parser: &Parser, datasource_id: &str) -> String {
    let with_datasource = parser
        .vector_toml
        .replace("${DATASOURCE_ID}", datasource_id);
    match parser.default_port {
        Some(port) => replace_listen_port_placeholders(&with_datasource, port),
        None => with_datasource,
    }
}

fn replace_listen_port_placeholders(input: &str, port: i32) -> String {
    let mut output = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find("${LISTEN_PORT") {
        output.push_str(&rest[..start]);
        let after_start = &rest[start..];
        if let Some(end) = after_start.find('}') {
            output.push_str(&port.to_string());
            rest = &after_start[end + 1..];
        } else {
            output.push_str(after_start);
            return output;
        }
    }
    output.push_str(rest);
    output
}

fn detect_address_collision(
    source_addresses: &mut BTreeMap<String, String>,
    component_name: &str,
    component_val: &Value,
    source_type: &str,
) -> Result<()> {
    let Some(address) = component_val
        .as_table()
        .and_then(|t| t.get("address"))
        .and_then(Value::as_str)
    else {
        return Ok(());
    };

    if let Some(existing) = source_addresses.insert(address.to_string(), component_name.to_string())
    {
        bail!(
            "source address collision on {address}: {existing} and {component_name} ({source_type})"
        );
    }

    Ok(())
}

/// Écrit le fichier cible de façon atomique (tmp + rename).
pub async fn write_atomic(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("creating parent dir {}", parent.display()))?;
    }

    let file_name = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("sources.toml");
    let tmp = path.with_file_name(format!(".{file_name}.{}.tmp", Uuid::now_v7()));
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
        let out = out.unwrap();
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
        let out = out.unwrap();
        assert!(out.contains("ds-acme-linux-syslog"));
        assert!(out.contains("ds-acme-network-opnsense"));
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
        let out = out.unwrap();
        assert!(out.contains("${LISTEN_PORT:-5141}"));
        assert!(out.contains("${TENANT_ID}"));
        assert!(out.contains("${QUICKWIT_ENDPOINT}"));
    }

    #[test]
    fn listen_port_is_substituted_from_default_port() {
        let mut p = fixture(
            "linux/syslog",
            r#"[sources.x]
type = "syslog"
address = "0.0.0.0:${LISTEN_PORT:-5141}""#,
        );
        p.default_port = Some(5141);
        let out = assemble_toml(&[p], "ds-acme").unwrap();
        assert!(out.contains("0.0.0.0:5141"));
        assert!(!out.contains("${LISTEN_PORT"));
    }

    #[test]
    fn duplicate_source_addresses_are_rejected() {
        let mut p1 = fixture(
            "linux/syslog",
            r#"[sources.a]
type = "syslog"
address = "0.0.0.0:${LISTEN_PORT:-5141}""#,
        );
        p1.default_port = Some(5141);
        let mut p2 = fixture(
            "linux/auth-log",
            r#"[sources.b]
type = "syslog"
address = "0.0.0.0:${LISTEN_PORT:-5142}""#,
        );
        p2.default_port = Some(5141);
        let err = assemble_toml(&[p1, p2], "ds-acme").unwrap_err();
        assert!(err.to_string().contains("source address collision"));
    }

    #[test]
    fn component_collisions_are_rejected() {
        let p1 = fixture(
            "linux/syslog",
            "[transforms.shared]\ntype = \"remap\"\nsource = \".\"",
        );
        let p2 = fixture(
            "linux/auth-log",
            "[transforms.shared]\ntype = \"remap\"\nsource = \".\"",
        );
        let err = assemble_toml(&[p1, p2], "ds-acme").unwrap_err();
        assert!(err.to_string().contains("component collision"));
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
