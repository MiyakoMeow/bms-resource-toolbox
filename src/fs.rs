//! File system operations.
//!
//! This module provides utilities for file manipulation,
//! directory syncing, and archive handling.

pub mod name;
pub mod pack_move;
pub mod rawpack;
pub mod sync;
pub mod utils;
pub mod walk;

/// Recursively check if a directory contains any non-empty files.
pub use pack_move::is_dir_having_file;
/// Synchronize files from source to destination based on a [`sync::SoftSyncPreset`].
pub use sync::{SYNC_PRESET_FOR_APPEND, sync_folder};
#[allow(unused_imports)]
/// Check if a directory contains a BMS chart file.
pub use walk::{has_chart_file, remove_empty_dirs};
