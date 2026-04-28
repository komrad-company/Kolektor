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
use khronika::configuration::{TelemetryConfiguration, TelemetryOutput};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = config::Cli::parse();
    khronika::intialize_logger(TelemetryConfiguration {
        level: cli.log_level.parse().unwrap_or(tracing::Level::INFO),
        output: TelemetryOutput::Remote {
            telemetry: String::new(),
        },
    });

    match cli.command {
        config::Command::Init(args) => init::run(args).await,
        config::Command::Serve(args) => serve::run(args).await,
        config::Command::Token(args) => token::run(args).await,
    }
}
