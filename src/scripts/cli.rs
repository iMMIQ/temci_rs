use clap::Parser;

#[derive(Parser, Debug)]
pub struct TemciCli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

pub async fn run(cli: TemciCli) -> anyhow::Result<()> {
    tracing::info!("temci CLI started with verbosity: {}", cli.verbose);
    Ok(())
}
