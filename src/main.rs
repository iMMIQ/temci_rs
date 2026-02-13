use clap::Parser;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod build;
mod report;
mod run;
mod scripts;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = temci::scripts::cli::TemciCli::parse();

    let filter_level = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(filter_level));

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting tracing subscriber failed");

    if let Err(e) = cli.run().await {
        tracing::error!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
