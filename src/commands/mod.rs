pub mod bigpack;
pub mod event;
pub mod folder;
pub mod media;
pub mod pack;
pub mod rawpack;

use crate::cli::{App, Command};

pub async fn dispatch(app: App) -> crate::Result<()> {
    match app.command {
        Command::Folder(cmd) => folder::handle(cmd).await,
        Command::Bigpack(cmd) => {
            bigpack::handle(cmd).await;
            Ok(())
        }
        Command::Event(cmd) => event::handle(cmd).await,
        Command::Media(cmd) => media::handle(cmd).await,
        Command::Rawpack(cmd) => rawpack::handle(cmd).await,
        Command::Pack(cmd) => pack::handle(cmd).await,
    }
}
