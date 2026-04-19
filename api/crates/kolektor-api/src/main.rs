mod config;
mod config_writer;
mod error;
mod init;
mod middleware;
mod routes;
mod serve;
mod state;
mod token;

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = config::Cli::parse();
    config::init_tracing(&cli);

    match cli.command {
        config::Command::Init(args) => init::run(args).await,
        config::Command::Serve(args) => serve::run(args).await,
        config::Command::Token(args) => token::run(args).await,
    }
}
