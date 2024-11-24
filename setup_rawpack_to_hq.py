import os

from bms_fs import is_dir_having_file
from bms_media.audio import (
    AUDIO_PRESET_FLAC,
    AUDIO_PRESET_FLAC_FFMPEG,
)
from scripts_bms_folder import (
    set_name_by_bms,
    transfer_audio,
    remove_unneed_media_file,
)
from scripts_rawpack import rawpack_unzip_to_bms_folder


def main():
    print("BMS Pack Generator by MiyakoMeow.")
    print(" - For Pack Create:")
    print(
        "Fast creating pack script, from: Raw Packs set numed, to: target bms folder."
    )
    print(
        "You need to set pack num before running this script, see scripts_rawpack/rawpack_set_num.py"
    )
    # Input
    print(" - Input 1: Pack dir path")
    pack_dir = input(">")
    if not os.path.isdir(pack_dir):
        print("Pack dir is not vaild dir.")
        return
    print(" - Input 2: BMS Cache Folder path")
    root_dir = input(">")
    if not os.path.isdir(root_dir):
        print("Root dir is not vaild dir.")
        return
    # Confirm
    confirm = input("Sure? [y/N]")
    if not confirm.lower().startswith("y"):
        return
    # Unzip
    print(f" > 1. Unzip packs from {pack_dir} to {root_dir}")
    cache_dir = os.path.join(root_dir, "CacheDir")
    rawpack_unzip_to_bms_folder.main(
        root_dir=root_dir,
        pack_dir=pack_dir,
        cache_dir=cache_dir,
    )
    if not is_dir_having_file(cache_dir):
        os.rmdir(cache_dir)
    # Syncing folder name
    print(" > 2. Setting dir names from BMS Files")
    set_name_by_bms.main(root_dir=root_dir)
    # Parse Audio
    print(" > 3. Parsing Audio... Phase 1: WAV -> FLAC")
    transfer_audio.main(
        root_dir=root_dir,
        input_ext=["wav"],
        transfer_mode=[AUDIO_PRESET_FLAC, AUDIO_PRESET_FLAC_FFMPEG],
        remove_origin_file_when_success=True,
        skip_on_fail=False,
    )
    # Remove Unneed Media File
    print(" > 4. Removing Unneed Files")
    remove_unneed_media_file.main(
        root_dir=root_dir, preset=remove_unneed_media_file.PRESET_NORMAL
    )


if __name__ == "__main__":
    main()
