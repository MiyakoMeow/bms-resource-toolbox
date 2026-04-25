//! File system operations.
//!
//! This module provides utilities for file manipulation,
//! directory syncing, and archive handling.

pub mod name;
pub mod pack_move;
pub mod rawpack;
pub mod sync;
pub mod walk;

pub use name::bms_dir_similarity;
pub use pack_move::is_dir_having_file;
pub use rawpack::{extract_numeric_to_bms_folder, move_out_files_in_folder_in_cache_dir};
pub use sync::{sync_folder, SYNC_PRESET_FOR_APPEND};
pub use walk::remove_empty_dirs;
