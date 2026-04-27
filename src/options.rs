//! CLI options and validation.
//!
//! This module provides command-line input handling
//! and external tool validation utilities.

pub mod input;
pub mod validator;
pub mod bms_folder;
pub mod bms_folder_bigpack;
pub mod bms_events;
pub mod bms_folder_media;
pub mod bms_folder_event;
pub mod rawpack;

pub use validator::{check_ffmpeg, check_flac, check_oggenc};
pub use bms_folder::{
    append_artist_name_by_bms, append_name_by_bms, copy_numbered_workdir_names,
    scan_folder_similar_folders, set_name_by_bms, undo_set_name,
};
pub use bms_folder_bigpack::{
    split_folders_with_first_char, undo_split_pack, move_works_in_pack,
    move_out_works, move_works_with_same_name, move_works_with_same_name_to_siblings,
    remove_zero_sized_media_files,
};
pub use bms_folder_event::{check_num_folder, create_num_folders, generate_work_info_table};
pub use bms_folder_media::{transfer_audio, transfer_video};
pub use rawpack::{unzip_numeric_to_bms_folder, unzip_with_name_to_bms_folder, set_file_num};
