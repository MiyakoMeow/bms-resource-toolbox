//! File system operations.
//!
//! This module provides utilities for file manipulation,
//! directory syncing, and archive handling.

/// Filename sanitization and BMS directory similarity comparison.
pub mod name;
/// File move, merge, and replacement logic.
pub mod pack_move;
/// Soft sync (selective directory synchronization with comparison presets).
pub mod sync;
/// Miscellaneous file system utilities (extension extraction, recursive copy).
pub mod utils;
/// Empty directory removal.
pub mod walk;

/// Recursively check if a directory contains any non-empty files.
#[allow(unused)]
pub use pack_move::is_dir_having_file;
/// Synchronize files from source to destination based on a [`sync::SoftSyncPreset`].
#[allow(unused)]
pub use sync::{SYNC_PRESET_FOR_APPEND, sync_folder};
/// Remove empty child directories.
#[allow(unused)]
pub use walk::remove_empty_dirs;
