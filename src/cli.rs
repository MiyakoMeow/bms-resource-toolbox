use std::path::PathBuf;

#[derive(clap::Parser)]
#[command(name = "bms-toolbox", about = "BMS Resource Toolbox")]
pub struct App {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    #[command(subcommand)]
    Folder(FolderCommand),
    #[command(subcommand)]
    Bigpack(BigpackCommand),
    #[command(subcommand)]
    Event(EventCommand),
    #[command(subcommand)]
    Media(MediaCommand),
    #[command(subcommand)]
    Rawpack(RawpackCommand),
    #[command(subcommand)]
    Pack(PackCommand),
}

#[derive(clap::Subcommand)]
pub enum FolderCommand {
    SetName { root_dir: PathBuf },
    AppendName { root_dir: PathBuf },
    AppendArtist { root_dir: PathBuf },
    CopyNames { src_dir: PathBuf, dst_dir: PathBuf },
    ScanSimilar {
        root_dir: PathBuf,
        #[arg(long, default_value = "0.7")]
        threshold: f64,
    },
    UndoRename { root_dir: PathBuf },
    CleanMedia { root_dir: PathBuf },
}

#[derive(clap::Subcommand)]
pub enum BigpackCommand {
    Split { root_dir: PathBuf },
    UndoSplit { dir_name: PathBuf },
    MoveWorks { from_dir: PathBuf, to_dir: PathBuf },
    MoveOut { root_dir: PathBuf },
    MoveSameName { from_dir: PathBuf, to_dir: PathBuf },
    MoveSameNameSiblings { dir: PathBuf },
    RemoveMedia {
        root_dir: PathBuf,
        #[arg(long, default_value = "0")]
        preset: usize,
    },
}

#[derive(clap::Subcommand)]
pub enum EventCommand {
    CheckNum { root_dir: PathBuf, count: u32 },
    CreateNum { root_dir: PathBuf, count: u32 },
}

#[derive(clap::Subcommand)]
pub enum MediaCommand {
    Audio {
        root_dir: PathBuf,
        #[arg(long)]
        mode: String,
    },
    Video {
        root_dir: PathBuf,
        #[arg(long)]
        preset: String,
        #[arg(long, default_value = "false")]
        auto_size: bool,
    },
}

#[derive(clap::Subcommand)]
pub enum RawpackCommand {
    UnzipNumeric {
        pack_dir: PathBuf,
        cache_dir: PathBuf,
        root_dir: PathBuf,
    },
    UnzipWithName {
        pack_dir: PathBuf,
        cache_dir: PathBuf,
        root_dir: PathBuf,
    },
    SetNum { dir: PathBuf },
}

#[derive(clap::Subcommand)]
pub enum PackCommand {
    RawToHq { root_dir: PathBuf },
    HqToLq { root_dir: PathBuf },
    SetupRawpackToHq { pack_dir: PathBuf, root_dir: PathBuf },
    UpdateRawpackToHq {
        pack_dir: PathBuf,
        root_dir: PathBuf,
        sync_dir: PathBuf,
    },
}
