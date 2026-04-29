//! File system operations.
//!
//! This module provides utilities for file manipulation,
//! directory syncing, and archive handling.

pub mod name;
pub mod pack_move;
pub mod rawpack;
pub mod sync;
pub mod walk;

pub use pack_move::is_dir_having_file;
pub use sync::{sync_folder, SYNC_PRESET_FOR_APPEND};
pub use walk::remove_empty_dirs;
