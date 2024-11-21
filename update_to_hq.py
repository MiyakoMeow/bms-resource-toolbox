import os

from bms_fs import get_bms_folder_dir, get_bms_pack_dir
from bms_media.audio import (
    AUDIO_PRESET_FLAC,
    AUDIO_PRESET_FLAC_FFMPEG,
)
from fs_sync import sync_folder
from fs_sync.preset import SYNC_PRESET_FOR_APPEND
from scripts_bms_folder import (
    set_name_by_another_folder,
    transfer_audio,
    remove_unneed_media_file,
)
from scripts_rawpack import rawpack_unzip_to_bms_folder


def main():
    print(
        "Fast update script, from: Raw Packs set numed, to: delta bms folder just for making pack update."
    )
    pack_dir = get_bms_pack_dir()
    root_dir = get_bms_folder_dir()
    print(
        "This script will use syncing dir for name syncing and file checking, but not modifying sync dir."
    )
    sync_dir = input("Input syncing dir:")
    if not os.path.isdir(sync_dir):
        print("Syncing dir is not vaild dir.")
        return
    confirm = input("Sure? [y/N]")
    if not confirm.lower().startswith("y"):
        return
    # Unzip
    print(f" > 1. Unzip dir name from {sync_dir} to {root_dir}")
    rawpack_unzip_to_bms_folder.main(
        root_dir=root_dir, pack_dir=pack_dir, cache_dir=root_dir
    )
    # Syncing folder name
    print(f" > 2. Syncing dir name from {sync_dir} to {root_dir}")
    set_name_by_another_folder.main(src_dir=sync_dir, dst_dir=root_dir)
    # Parse Audio
    print(" > 3. Parsing Audio... Phase 1: WAV -> FLAC")
    transfer_audio.main(
        root_dir=root_dir,
        input_ext=["wav"],
        transfer_mode=[AUDIO_PRESET_FLAC, AUDIO_PRESET_FLAC_FFMPEG],
        remove_origin_file=True,
        skip_on_fail=False,
    )
    # Remove Unneed Media File
    print(" > 4. Removing Unneed Files")
    remove_unneed_media_file.main(
        root_dir, preset=remove_unneed_media_file.PRESET_NORMAL
    )
    # Soft syncing
    print(f" > 5. Syncing dir files from {sync_dir} to {root_dir}")
    sync_folder(src_dir=root_dir, dst_dir=sync_dir, preset=SYNC_PRESET_FOR_APPEND)


if __name__ == "__main__":
    main()
