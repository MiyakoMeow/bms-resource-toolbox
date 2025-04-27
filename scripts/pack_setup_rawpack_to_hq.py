import os

from fs.move import is_dir_having_file
from media.audio import (
    AUDIO_PRESET_FLAC,
    AUDIO_PRESET_FLAC_FFMPEG,
    bms_folder_transfer_audio,
)

from options.bms_folder import append_name_by_bms
from options.bms_folder_bigpack import (
    REMOVE_MEDIA_RULE_ORAJA,
    remove_unneed_media_files,
)
import rawpack_unzip_numeric_to_bms_folder


def main():
    print("BMS Pack Generator by MiyakoMeow.")
    print(" - For Pack Create:")
    print(
        "Fast creating pack script, from: Raw Packs set numed, to: target bms folder."
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
    file_id_names = rawpack_unzip_numeric_to_bms_folder.get_num_set_file_names(pack_dir)
    print(" -- There are packs in pack_dir:")
    for file_name in file_id_names:
        print(f" > {file_name}")
    # Input 2
    print(" - Input 2: BMS Cache Folder path. (Input a dir path that NOT exists)")
    root_dir = input(">")
    if os.path.isdir(root_dir):
        print("Root dir is an existing dir.")
        return
    # Confirm
    confirm = input("Sure? [y/N]")
    if not confirm.lower().startswith("y"):
        return
    # Setup
    os.makedirs(root_dir, exist_ok=False)
    # Unzip
    print(f" > 1. Unzip packs from {pack_dir} to {root_dir}")
    cache_dir = os.path.join(root_dir, "CacheDir")
    rawpack_unzip_numeric_to_bms_folder.main(
        root_dir=root_dir,
        pack_dir=pack_dir,
        cache_dir=cache_dir,
    )
    if not is_dir_having_file(cache_dir):
        os.rmdir(cache_dir)
    # Syncing folder name
    print(" > 2. Setting dir names from BMS Files")
    append_name_by_bms(root_dir=root_dir)
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


if __name__ == "__main__":
    main()
