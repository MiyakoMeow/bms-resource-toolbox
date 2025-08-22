use clap::Parser;

use be_music_cabinet::{Cli, run_command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    run_command(&cli.command).await?;

    Ok(())
}
