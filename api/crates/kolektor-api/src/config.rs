use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about = "Kolektor — API REST + init binaire")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Niveau de log
    #[arg(long, default_value = "info", env = "LOG_LEVEL", global = true)]
    pub log_level: String,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Migrations + seed catalog + écriture initiale du fichier Vector
    Init(InitArgs),
    /// Démarre l'API HTTP
    Serve(ServeArgs),
    /// Gestion des tokens d'API (bootstrap)
    Token(TokenArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct TokenArgs {
    #[command(subcommand)]
    pub command: TokenCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum TokenCommand {
    /// Crée un nouveau token et affiche sa valeur (une seule fois).
    Create(TokenCreateArgs),
    /// Liste les tokens (sans exposer les secrets).
    List(TokenListArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct TokenCreateArgs {
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,

    #[arg(long)]
    pub name: String,

    #[arg(long, default_value = "acme")]
    pub tenant_id: String,
}

#[derive(Parser, Debug, Clone)]
pub struct TokenListArgs {
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,
}

#[derive(Parser, Debug, Clone)]
pub struct InitArgs {
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,

    #[arg(long, default_value = "/etc/vector/catalog", env = "CATALOG_DIR")]
    pub catalog_dir: String,

    #[arg(
        long,
        default_value = "/etc/vector/kolektor/sources.toml",
        env = "VECTOR_OUTPUT"
    )]
    pub vector_output: String,
}

#[derive(Parser, Debug, Clone)]
pub struct ServeArgs {
    #[arg(long, default_value = "0.0.0.0:8080", env = "LISTEN_ADDR")]
    pub listen_addr: String,

    #[arg(long, env = "TENANT_ID")]
    pub tenant_id: Option<String>,

    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,

    #[arg(long, default_value = "10", env = "DATABASE_MAX_CONNECTIONS")]
    pub database_max_connections: u32,

    #[arg(
        long,
        default_value = "/etc/vector/kolektor/sources.toml",
        env = "VECTOR_OUTPUT"
    )]
    pub vector_output: String,
}
