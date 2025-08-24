use clap::Parser;

use be_music_cabinet::{Cli, run_command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 CLI 日志，使日志原样输出到终端
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    smol::block_on(async {
        let cli = Cli::parse();
        run_command(&cli.command).await?;
        Ok(())
    })
}
