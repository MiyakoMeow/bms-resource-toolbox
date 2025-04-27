import os

from media.audio import (
    AUDIO_PRESET_FLAC,
    AUDIO_PRESET_FLAC_FFMPEG,
    bms_folder_transfer_audio,
)
from fs.sync import sync_folder, SYNC_PRESET_FOR_APPEND
from fs.rawpack import get_num_set_file_names

from options.bms_folder import copy_numbered_workdir_names
from options.bms_folder_bigpack import (
    REMOVE_MEDIA_RULE_ORAJA,
    remove_unneed_media_files,
)
import remove_empty_folder
import rawpack_unzip_numeric_to_bms_folder


def main():
    print("BMS Pack Generator by MiyakoMeow.")
    print(" - For Pack Update:")
    print(
        "Fast update script, from: Raw Packs set numed, to: delta bms folder just for making pack update."
    )
    print(
        "You need to set pack num before running this script, see scripts_rawpack/rawpack_set_num.py"
    )
    # Input 1
    print(" - Input 1: Pack dir path")
    pack_dir = input(">")
    if not os.path.isdir(pack_dir):
        print("Pack dir is not vaild dir.")
        return
    # Print Packs
    file_id_names = get_num_set_file_names(pack_dir)
    print(" -- There are packs in pack_dir:")
    for file_name in file_id_names:
        print(f" > {file_name}")
    # Input 2
    print(" - Input 2: BMS Cache Folder path. (Input a dir path that NOT exists)")
    root_dir = input(">")
    if os.path.isdir(root_dir):
        print("Root dir is an existing dir.")
        return
    # Input 3
    print(
        " - Input 3: Already exists BMS Folder path. (Input a dir path that ALREADY exists)"
    )
    print("This script will use this dir, just for name syncing and file checking.")
    sync_dir = input(">")
    if not os.path.isdir(sync_dir):
        print("Syncing dir is not vaild dir.")
        return
    # Confirm
    confirm = input("Sure? [y/N]")
    if not confirm.lower().startswith("y"):
        return
    # Setup
    os.makedirs(root_dir, exist_ok=False)
    # Unzip
    print(f" > 1. Unzip packs from {pack_dir} to {root_dir}")
    rawpack_unzip_numeric_to_bms_folder.main(
        root_dir=root_dir,
        pack_dir=pack_dir,
        cache_dir=os.path.join(root_dir, "CacheDir"),
    )
    # Syncing folder name
    print(f" > 2. Syncing dir name from {sync_dir} to {root_dir}")
    copy_numbered_workdir_names(sync_dir, root_dir)
    # Parse Audio
    print(" > 3. Parsing Audio... Phase 1: WAV -> FLAC")
    bms_folder_transfer_audio(
        root_dir=root_dir,
        input_ext=["wav"],
        transfer_mode=[AUDIO_PRESET_FLAC, AUDIO_PRESET_FLAC_FFMPEG],
        remove_origin_file_when_success=True,
        skip_on_fail=False,
    )
    # Remove Unneed Media File
    print(" > 4. Removing Unneed Files")
    remove_unneed_media_files(root_dir=root_dir, rules=REMOVE_MEDIA_RULE_ORAJA)
    # Soft syncing
    print(f" > 5. Syncing dir files from {sync_dir} to {root_dir}")
    sync_folder(src_dir=root_dir, dst_dir=sync_dir, preset=SYNC_PRESET_FOR_APPEND)
    # Remove Empty folder
    print(f" > 6. Remove empty folder in {root_dir}")
    remove_empty_folder.main(parent_dir=root_dir)


if __name__ == "__main__":
    main()
