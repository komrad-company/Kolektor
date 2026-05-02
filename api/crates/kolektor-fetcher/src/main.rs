use std::{
    collections::BTreeMap,
    env,
    io::Read,
    path::{Component, Path},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, SecondsFormat, TimeDelta, Utc};
use clap::{Parser, Subcommand};
use flate2::read::GzDecoder;
use futures::TryStreamExt;
use kolektor_common::{db, models::Fetcher};
use object_store::{ObjectStore, aws::AmazonS3Builder, path::Path as ObjectPath};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use url::Url;
use uuid::Uuid;

const FETCHER_OUTPUT_BASE_DIR: &str = "/var/lib/kolektor/fetcher";

#[derive(Parser, Debug)]
#[command(version, about = "Kolektor pull fetcher for cloud/SaaS logs")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    #[arg(long, default_value = "info", env = "LOG_LEVEL", global = true)]
    log_level: String,

    #[arg(long, default_value = "json", env = "LOG_FORMAT", global = true)]
    log_format: String,
}

#[derive(Subcommand, Debug)]
enum Command {
    Run(RunArgs),
}

#[derive(Parser, Debug, Clone)]
struct RunArgs {
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    #[arg(long, default_value = "5", env = "DATABASE_MAX_CONNECTIONS")]
    database_max_connections: u32,

    #[arg(long, default_value = "30", env = "FETCHER_POLL_SECONDS")]
    poll_seconds: u64,

    #[arg(long, env = "FETCHER_ONCE")]
    once: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct FetcherState {
    #[serde(default)]
    cursors: BTreeMap<String, String>,
    last_attempt_at: Option<DateTime<Utc>>,
    last_success_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct MicrosoftGraphConfig {
    tenant_id: String,
    client_id: String,
    client_secret: Option<String>,
    client_secret_env: Option<String>,
    #[serde(default = "default_graph_kind")]
    kind: String,
    #[serde(default = "default_graph_base_url")]
    graph_base_url: String,
    #[serde(default = "default_authority_host")]
    authority_host: String,
    #[serde(default = "default_lookback_minutes")]
    lookback_minutes: i64,
    #[serde(default = "default_safety_lag_seconds")]
    safety_lag_seconds: i64,
    max_pages: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct Microsoft365Config {
    tenant_id: String,
    client_id: String,
    client_secret: Option<String>,
    client_secret_env: Option<String>,
    #[serde(default = "default_m365_api_base_url")]
    api_base_url: String,
    #[serde(default = "default_authority_host")]
    authority_host: String,
    #[serde(default = "default_m365_content_types")]
    content_types: Vec<String>,
    #[serde(default)]
    ensure_subscriptions: bool,
    publisher_identifier: Option<String>,
    #[serde(default = "default_lookback_minutes")]
    lookback_minutes: i64,
    #[serde(default = "default_safety_lag_seconds")]
    safety_lag_seconds: i64,
}

#[derive(Debug, Deserialize)]
struct S3Config {
    bucket: String,
    #[serde(default)]
    prefix: String,
    region: Option<String>,
    endpoint: Option<String>,
    access_key_id: Option<String>,
    access_key_id_env: Option<String>,
    secret_access_key: Option<String>,
    secret_access_key_env: Option<String>,
    session_token: Option<String>,
    session_token_env: Option<String>,
    #[serde(default)]
    force_path_style: bool,
    max_objects: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct GraphPage {
    #[serde(rename = "value")]
    value: Vec<Value>,
    #[serde(rename = "@odata.nextLink")]
    next_link: Option<String>,
}

#[derive(Debug, Deserialize)]
struct M365Content {
    #[serde(rename = "contentUri")]
    content_uri: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(&cli);

    match cli.command {
        Command::Run(args) => run(args).await,
    }
}

async fn run(args: RunArgs) -> Result<()> {
    let pool = db::connect(&args.database_url, args.database_max_connections).await?;
    let http = Client::builder()
        .user_agent("kolektor-fetcher/0.1")
        .timeout(Duration::from_secs(60))
        .build()?;

    loop {
        let fetchers = load_enabled_fetchers(&pool).await?;
        for fetcher in fetchers {
            if !args.once && !is_due(&fetcher) {
                continue;
            }
            if let Err(err) = run_one(&pool, &http, fetcher.clone()).await {
                tracing::error!(
                    fetcher_id = %fetcher.id,
                    provider = %fetcher.provider,
                    error = %err,
                    "fetcher failed"
                );
                record_failure(&pool, fetcher.id, &err.to_string()).await?;
            }
        }

        if args.once {
            break;
        }
        tokio::time::sleep(Duration::from_secs(args.poll_seconds)).await;
    }

    Ok(())
}

async fn load_enabled_fetchers(pool: &PgPool) -> Result<Vec<Fetcher>> {
    sqlx::query_as::<_, Fetcher>(
        "SELECT * FROM kolektor.fetchers WHERE enabled = true ORDER BY provider, name",
    )
    .fetch_all(pool)
    .await
    .context("loading enabled fetchers")
}

fn is_due(fetcher: &Fetcher) -> bool {
    let Some(last_attempt_at) = fetcher.last_attempt_at else {
        return true;
    };
    let elapsed = Utc::now().signed_duration_since(last_attempt_at);
    elapsed >= TimeDelta::seconds(fetcher.interval_seconds.into())
}

async fn run_one(pool: &PgPool, http: &Client, fetcher: Fetcher) -> Result<()> {
    record_attempt(pool, fetcher.id).await?;
    let mut state = parse_state(&fetcher.state)?;

    let fetched = match fetcher.provider.as_str() {
        "microsoft_graph" => fetch_microsoft_graph(http, &fetcher, &mut state).await?,
        "microsoft365_management" => fetch_microsoft365(http, &fetcher, &mut state).await?,
        "s3" => fetch_s3(&fetcher, &mut state).await?,
        other => bail!("unsupported provider {other}"),
    };

    state.last_success_at = Some(Utc::now());
    record_success(pool, fetcher.id, &state).await?;

    tracing::info!(
        fetcher_id = %fetcher.id,
        provider = %fetcher.provider,
        fetched,
        output_path = %fetcher.output_path,
        "fetcher completed"
    );

    Ok(())
}

async fn fetch_microsoft_graph(
    http: &Client,
    fetcher: &Fetcher,
    state: &mut FetcherState,
) -> Result<usize> {
    let cfg: MicrosoftGraphConfig =
        serde_json::from_value(fetcher.config.clone()).context("invalid microsoft_graph config")?;
    let token = microsoft_token(
        http,
        &cfg.authority_host,
        &cfg.tenant_id,
        &cfg.client_id,
        &client_secret(cfg.client_secret, cfg.client_secret_env)?,
        &format!("{}/.default", cfg.graph_base_url.trim_end_matches('/')),
    )
    .await?;

    let (path, time_field) = match cfg.kind.as_str() {
        "directory_audits" | "directoryAudits" => ("auditLogs/directoryAudits", "activityDateTime"),
        "signins" | "signIns" => ("auditLogs/signIns", "createdDateTime"),
        other => bail!("unsupported microsoft_graph kind {other}"),
    };

    let cursor_key = format!("microsoft_graph:{}", cfg.kind);
    let start = cursor_or_lookback(state, &cursor_key, cfg.lookback_minutes)?;
    let end = Utc::now() - TimeDelta::seconds(cfg.safety_lag_seconds.max(0));

    if start >= end {
        return Ok(0);
    }

    let filter = format!(
        "{time_field} ge {} and {time_field} le {}",
        format_graph_time(start),
        format_graph_time(end)
    );
    let mut url = Url::parse(&format!(
        "{}/v1.0/{path}",
        cfg.graph_base_url.trim_end_matches('/')
    ))?;
    url.query_pairs_mut()
        .append_pair("$filter", &filter)
        .append_pair("$orderby", &format!("{time_field} asc"))
        .append_pair("$top", "1000");

    let mut fetched = 0usize;
    let mut max_seen = start;
    let mut page_count = 0usize;
    let mut next_url = Some(url.to_string());

    while let Some(page_url) = next_url {
        page_count += 1;
        if cfg.max_pages.is_some_and(|max| page_count > max) {
            break;
        }

        let page: GraphPage = http
            .get(&page_url)
            .bearer_auth(&token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        append_values(&fetcher.output_path, &page.value).await?;
        fetched += page.value.len();

        for event in &page.value {
            if let Some(ts) = event_time(event, time_field) {
                max_seen = max_seen.max(ts);
            }
        }

        next_url = page.next_link;
    }

    let next_cursor = if fetched > 0 {
        max_seen + TimeDelta::milliseconds(1)
    } else {
        end
    };
    state
        .cursors
        .insert(cursor_key, format_graph_time(next_cursor));

    Ok(fetched)
}

async fn fetch_microsoft365(
    http: &Client,
    fetcher: &Fetcher,
    state: &mut FetcherState,
) -> Result<usize> {
    let cfg: Microsoft365Config = serde_json::from_value(fetcher.config.clone())
        .context("invalid microsoft365_management config")?;
    let token = microsoft_token(
        http,
        &cfg.authority_host,
        &cfg.tenant_id,
        &cfg.client_id,
        &client_secret(cfg.client_secret, cfg.client_secret_env)?,
        "https://manage.office.com/.default",
    )
    .await?;

    let mut fetched = 0usize;
    let base = cfg.api_base_url.trim_end_matches('/');
    for content_type in &cfg.content_types {
        if cfg.ensure_subscriptions {
            ensure_m365_subscription(
                http,
                base,
                &cfg.tenant_id,
                content_type,
                &token,
                cfg.publisher_identifier.as_deref(),
            )
            .await?;
        }

        let cursor_key = format!("m365:{content_type}");
        let start = cursor_or_lookback(state, &cursor_key, cfg.lookback_minutes)?;
        let available_end = Utc::now() - TimeDelta::seconds(cfg.safety_lag_seconds.max(0));
        let end = available_end.min(start + TimeDelta::hours(24));
        if start >= end {
            continue;
        }

        let mut url = Url::parse(&format!(
            "{base}/api/v1.0/{}/activity/feed/subscriptions/content",
            cfg.tenant_id
        ))?;
        url.query_pairs_mut()
            .append_pair("contentType", content_type)
            .append_pair("startTime", &format_graph_time(start))
            .append_pair("endTime", &format_graph_time(end));
        append_publisher_identifier(&mut url, cfg.publisher_identifier.as_deref());

        let mut next_page = Some(url.to_string());
        while let Some(page_url) = next_page {
            let response = http
                .get(page_url)
                .bearer_auth(&token)
                .send()
                .await?
                .error_for_status()?;
            next_page = response
                .headers()
                .get("NextPageUri")
                .and_then(|value| value.to_str().ok())
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned);

            let contents: Vec<M365Content> = response.json().await?;

            for content in contents {
                let blob: Value = http
                    .get(&content.content_uri)
                    .bearer_auth(&token)
                    .send()
                    .await?
                    .error_for_status()?
                    .json()
                    .await?;

                match blob {
                    Value::Array(values) => {
                        fetched += values.len();
                        append_values(&fetcher.output_path, &values).await?;
                    }
                    value => {
                        fetched += 1;
                        append_values(&fetcher.output_path, &[value]).await?;
                    }
                }
            }
        }

        state.cursors.insert(cursor_key, format_graph_time(end));
    }

    Ok(fetched)
}

async fn ensure_m365_subscription(
    http: &Client,
    base: &str,
    tenant_id: &str,
    content_type: &str,
    token: &str,
    publisher_identifier: Option<&str>,
) -> Result<()> {
    let mut url = Url::parse(&format!(
        "{base}/api/v1.0/{tenant_id}/activity/feed/subscriptions/start"
    ))?;
    url.query_pairs_mut()
        .append_pair("contentType", content_type);
    append_publisher_identifier(&mut url, publisher_identifier);

    let response = http.post(url).bearer_auth(token).send().await?;
    if response.status().is_success()
        || response.status() == StatusCode::BAD_REQUEST
        || response.status() == StatusCode::CONFLICT
    {
        Ok(())
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        bail!("failed to ensure M365 subscription {content_type}: {status} {body}")
    }
}

fn append_publisher_identifier(url: &mut Url, publisher_identifier: Option<&str>) {
    if let Some(publisher_identifier) = publisher_identifier
        && !publisher_identifier.is_empty()
    {
        url.query_pairs_mut()
            .append_pair("PublisherIdentifier", publisher_identifier);
    }
}

async fn fetch_s3(fetcher: &Fetcher, state: &mut FetcherState) -> Result<usize> {
    let cfg: S3Config =
        serde_json::from_value(fetcher.config.clone()).context("invalid s3 config")?;

    let mut builder = AmazonS3Builder::new().with_bucket_name(&cfg.bucket);
    if let Some(region) = &cfg.region {
        builder = builder.with_region(region);
    }
    if let Some(endpoint) = &cfg.endpoint {
        builder = builder.with_endpoint(endpoint);
    }
    if cfg.force_path_style {
        builder = builder.with_virtual_hosted_style_request(false);
    }
    if let Some(access_key_id) = optional_secret(cfg.access_key_id, cfg.access_key_id_env)? {
        builder = builder.with_access_key_id(access_key_id);
    }
    if let Some(secret_access_key) =
        optional_secret(cfg.secret_access_key, cfg.secret_access_key_env)?
    {
        builder = builder.with_secret_access_key(secret_access_key);
    }
    if let Some(session_token) = optional_secret(cfg.session_token, cfg.session_token_env)? {
        builder = builder.with_token(session_token);
    }

    let store = builder.build()?;
    let prefix = if cfg.prefix.is_empty() {
        None
    } else {
        Some(ObjectPath::from(cfg.prefix.as_str()))
    };
    let mut stream = store.list(prefix.as_ref());
    let last_key = state.cursors.get("s3:last_key").cloned();
    let mut fetched = 0usize;
    let mut max_key = last_key.clone();

    while let Some(meta) = stream.try_next().await? {
        let key = meta.location.to_string();
        if last_key.as_ref().is_some_and(|last| key <= *last) {
            continue;
        }
        if cfg.max_objects.is_some_and(|max| fetched >= max) {
            break;
        }

        let bytes = store.get(&meta.location).await?.bytes().await?;
        let lines = decode_s3_object(&key, bytes.as_ref())?;
        append_lines(&fetcher.output_path, &lines).await?;
        fetched += lines.len();
        max_key = Some(key);
    }

    if let Some(max_key) = max_key {
        state.cursors.insert("s3:last_key".to_string(), max_key);
    }

    Ok(fetched)
}

fn decode_s3_object(key: &str, bytes: &[u8]) -> Result<Vec<String>> {
    let mut decoded = Vec::new();
    if key.ends_with(".gz") {
        let mut decoder = GzDecoder::new(bytes);
        decoder.read_to_end(&mut decoded)?;
    } else {
        decoded.extend_from_slice(bytes);
    }

    let text = String::from_utf8(decoded).context("S3 object is not UTF-8 after optional gzip")?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    if trimmed.starts_with('[') {
        let values: Vec<Value> = serde_json::from_str(trimmed)?;
        return values
            .into_iter()
            .map(|value| serde_json::to_string(&value).map_err(Into::into))
            .collect();
    }

    Ok(text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

async fn microsoft_token(
    http: &Client,
    authority_host: &str,
    tenant_id: &str,
    client_id: &str,
    client_secret: &str,
    scope: &str,
) -> Result<String> {
    let url = format!(
        "{}/{tenant_id}/oauth2/v2.0/token",
        authority_host.trim_end_matches('/')
    );
    let response: TokenResponse = http
        .post(url)
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("scope", scope),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response.access_token)
}

fn client_secret(secret: Option<String>, env_name: Option<String>) -> Result<String> {
    optional_secret(secret, env_name)?.context("missing client_secret or client_secret_env")
}

fn optional_secret(secret: Option<String>, env_name: Option<String>) -> Result<Option<String>> {
    if let Some(secret) = secret {
        return Ok(Some(secret));
    }
    if let Some(env_name) = env_name {
        return Ok(Some(
            env::var(&env_name).with_context(|| format!("missing env var {env_name}"))?,
        ));
    }
    Ok(None)
}

fn parse_state(value: &Value) -> Result<FetcherState> {
    if value.is_null() {
        Ok(FetcherState::default())
    } else {
        serde_json::from_value(value.clone()).context("invalid fetcher state")
    }
}

fn cursor_or_lookback(
    state: &FetcherState,
    cursor_key: &str,
    lookback_minutes: i64,
) -> Result<DateTime<Utc>> {
    if let Some(cursor) = state.cursors.get(cursor_key) {
        return DateTime::parse_from_rfc3339(cursor)
            .map(|dt| dt.with_timezone(&Utc))
            .with_context(|| format!("invalid cursor {cursor_key}={cursor:?}"));
    }
    Ok(Utc::now() - TimeDelta::minutes(lookback_minutes.max(1)))
}

fn event_time(event: &Value, field: &str) -> Option<DateTime<Utc>> {
    event
        .get(field)
        .and_then(Value::as_str)
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn format_graph_time(ts: DateTime<Utc>) -> String {
    ts.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

async fn append_values(path: &str, values: &[Value]) -> Result<()> {
    let lines: Result<Vec<_>, _> = values.iter().map(serde_json::to_string).collect();
    append_lines(path, &lines?).await
}

async fn append_lines(path: &str, lines: &[String]) -> Result<()> {
    if lines.is_empty() {
        return Ok(());
    }

    validate_output_path(path)?;

    if let Some(parent) = Path::new(path).parent()
        && !parent.as_os_str().is_empty()
    {
        tokio::fs::create_dir_all(parent).await?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .with_context(|| format!("opening output path {path}"))?;

    for line in lines {
        file.write_all(line.as_bytes()).await?;
        file.write_all(b"\n").await?;
    }
    file.flush().await?;
    Ok(())
}

fn validate_output_path(output_path: &str) -> Result<()> {
    let path = Path::new(output_path);
    let base = Path::new(FETCHER_OUTPUT_BASE_DIR);
    let has_parent_dir = path
        .components()
        .any(|component| matches!(component, Component::ParentDir));

    if path.is_absolute()
        && path.starts_with(base)
        && path != base
        && !has_parent_dir
        && path.file_name().is_some()
    {
        Ok(())
    } else {
        bail!("output_path must be an absolute file path under {FETCHER_OUTPUT_BASE_DIR}")
    }
}

async fn record_attempt(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query(
        "UPDATE kolektor.fetchers
         SET last_attempt_at = now(), updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn record_success(pool: &PgPool, id: Uuid, state: &FetcherState) -> Result<()> {
    sqlx::query(
        "UPDATE kolektor.fetchers
         SET state = $2, last_success_at = now(), last_error = NULL, updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(serde_json::to_value(state)?)
    .execute(pool)
    .await?;
    Ok(())
}

async fn record_failure(pool: &PgPool, id: Uuid, error: &str) -> Result<()> {
    sqlx::query(
        "UPDATE kolektor.fetchers
         SET last_error = $2, updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(error)
    .execute(pool)
    .await?;
    Ok(())
}

fn default_graph_kind() -> String {
    "signins".to_string()
}

fn default_graph_base_url() -> String {
    "https://graph.microsoft.com".to_string()
}

fn default_authority_host() -> String {
    "https://login.microsoftonline.com".to_string()
}

fn default_m365_api_base_url() -> String {
    "https://manage.office.com".to_string()
}

fn default_m365_content_types() -> Vec<String> {
    vec![
        "Audit.AzureActiveDirectory".to_string(),
        "Audit.Exchange".to_string(),
        "Audit.SharePoint".to_string(),
        "Audit.General".to_string(),
    ]
}

fn default_lookback_minutes() -> i64 {
    15
}

fn default_safety_lag_seconds() -> i64 {
    120
}

fn init_tracing(cli: &Cli) {
    let env_filter = EnvFilter::try_new(&cli.log_level).unwrap_or_else(|_| EnvFilter::new("info"));
    let registry = tracing_subscriber::registry().with(env_filter);
    if cli.log_format == "json" {
        registry.with(fmt::layer().json()).init();
    } else {
        registry.with(fmt::layer().pretty()).init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn decodes_ndjson_s3_object() {
        let lines = decode_s3_object(
            "logs/a.json",
            br#"{"a":1}
{"b":2}
"#,
        )
        .unwrap();

        assert_eq!(lines, vec![r#"{"a":1}"#, r#"{"b":2}"#]);
    }

    #[test]
    fn decodes_json_array_s3_object() {
        let lines = decode_s3_object("logs/a.json", br#"[{"a":1},{"b":2}]"#).unwrap();

        assert_eq!(lines, vec![r#"{"a":1}"#, r#"{"b":2}"#]);
    }

    #[test]
    fn cursor_uses_existing_state() {
        let mut cursors = BTreeMap::new();
        cursors.insert("x".to_string(), "2026-04-22T10:00:00Z".to_string());
        let state = FetcherState {
            cursors,
            ..Default::default()
        };

        assert_eq!(
            cursor_or_lookback(&state, "x", 15).unwrap(),
            DateTime::parse_from_rfc3339("2026-04-22T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        );
    }

    #[test]
    fn graph_event_time_reads_expected_field() {
        let event = json!({"createdDateTime": "2026-04-22T10:00:00Z"});

        assert_eq!(
            event_time(&event, "createdDateTime").unwrap(),
            DateTime::parse_from_rfc3339("2026-04-22T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        );
    }

    #[test]
    fn api_config_defaults_are_stable() {
        let cfg: MicrosoftGraphConfig = serde_json::from_value(json!({
            "tenant_id": "tenant",
            "client_id": "client",
            "client_secret_env": "SECRET"
        }))
        .unwrap();

        assert_eq!(cfg.kind, "signins");
        assert_eq!(cfg.graph_base_url, "https://graph.microsoft.com");
        assert_eq!(cfg.lookback_minutes, 15);
    }

    #[test]
    fn object_config_supports_s3_compatible_endpoint() {
        let cfg: S3Config = serde_json::from_value(json!({
            "bucket": "logs",
            "prefix": "m365/",
            "endpoint": "https://s3.example.test",
            "force_path_style": true
        }))
        .unwrap();

        assert_eq!(cfg.bucket, "logs");
        assert!(cfg.force_path_style);
    }

    #[test]
    fn output_path_must_stay_under_fetcher_dir() {
        assert!(validate_output_path("/var/lib/kolektor/fetcher/microsoft-entra.jsonl").is_ok());
        assert!(validate_output_path("/tmp/microsoft-entra.jsonl").is_err());
        assert!(validate_output_path("/var/lib/kolektor/fetcher/../escape.jsonl").is_err());
    }

    #[test]
    fn publisher_identifier_is_appended_when_present() {
        let mut url = Url::parse("https://manage.office.com/api/v1.0/t/activity/feed/subscriptions/content?contentType=Audit.General").unwrap();

        append_publisher_identifier(&mut url, Some("publisher"));

        assert!(url.as_str().contains("PublisherIdentifier=publisher"));
    }

    #[test]
    fn state_null_is_default() {
        let state = parse_state(&Value::Null).unwrap();

        assert!(state.cursors.is_empty());
    }

    #[test]
    fn format_graph_time_uses_utc_z() {
        let dt = DateTime::parse_from_rfc3339("2026-04-22T10:00:00+00:00")
            .unwrap()
            .with_timezone(&Utc);

        assert_eq!(format_graph_time(dt), "2026-04-22T10:00:00Z");
    }

    #[test]
    fn format_graph_time_preserves_millisecond_cursor() {
        let dt = DateTime::parse_from_rfc3339("2026-04-22T10:00:00.001+00:00")
            .unwrap()
            .with_timezone(&Utc);

        assert_eq!(format_graph_time(dt), "2026-04-22T10:00:00.001Z");
    }

    #[test]
    fn optional_secret_prefers_literal_value() {
        assert_eq!(
            optional_secret(Some("literal".to_string()), Some("NOPE".to_string())).unwrap(),
            Some("literal".to_string())
        );
    }

    #[test]
    fn missing_optional_secret_is_none() {
        assert_eq!(optional_secret(None, None).unwrap(), None);
    }

    #[test]
    fn client_secret_requires_value() {
        assert!(client_secret(None, None).is_err());
    }

    #[test]
    fn due_when_never_attempted() {
        let mut fetcher = test_fetcher();
        fetcher.last_attempt_at = None;

        assert!(is_due(&fetcher));
    }

    #[test]
    fn not_due_inside_interval() {
        let mut fetcher = test_fetcher();
        fetcher.last_attempt_at = Some(Utc::now());
        fetcher.interval_seconds = 300;

        assert!(!is_due(&fetcher));
    }

    fn test_fetcher() -> Fetcher {
        Fetcher {
            id: Uuid::now_v7(),
            name: "test".to_string(),
            provider: "s3".to_string(),
            parser_source_type: "cloud/test".to_string(),
            enabled: true,
            interval_seconds: 300,
            output_path: "/tmp/test.jsonl".to_string(),
            config: json!({}),
            state: json!({}),
            last_attempt_at: None,
            last_success_at: None,
            last_error: None,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
