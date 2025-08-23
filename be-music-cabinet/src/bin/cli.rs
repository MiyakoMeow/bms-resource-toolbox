use clap::Parser;

use be_music_cabinet::{Cli, run_command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    smol::block_on(async {
        let cli = Cli::parse();
        run_command(&cli.command).await?;
        Ok(())
    })
}
