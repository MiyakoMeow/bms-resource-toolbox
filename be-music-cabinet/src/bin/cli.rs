use clap::Parser;

use be_music_cabinet::{Cli, run_command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize CLI logger to output logs as-is to terminal
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    smol::block_on(async {
        let cli = Cli::parse();
        run_command(&cli.command).await?;
        Ok(())
    })
}
