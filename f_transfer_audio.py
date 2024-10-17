import os
from typing import Optional, Tuple, List

from bms_fs import get_bms_folder_dir
from bms_media import transfer_audio_by_format_in_dir

MODES: List[Tuple[str, Tuple[str, str, str, Optional[str]]]] = [
    ("WAV to FLAC", ("wav", "flac", "flac", None)),
    ("FLAC to OGG", ("flac", "ogg", "ogg", "-ab 320k")),
    ("FLAC to WAV", ("flac", "wav", "wav", None)),
    # Above is oftenly useful
    ("OGG to FLAC", ("ogg", "flac", "flac", None)),
    ("OGG to WAV", ("ogg", "wav", "wav", None)),
    ("WAV to OGG", ("wav", "ogg", "ogg", "-ab 320k")),
]


def main(
    transfer_mode: Optional[Tuple[str, str, str, Optional[str]]],
    remove_origin_file: bool = True,
):
    root_dir = get_bms_folder_dir()

    # Select Modes
    if transfer_mode is None:
        for i, (mode_str, mode_args) in enumerate(MODES):
            print(f"- {i}: {mode_str} ({mode_args})")
        selection = int(input("Select Mode (Type numbers above):"))
        transfer_mode = MODES[selection][1]

    for bms_dir_name in os.listdir(root_dir):
        bms_dir_path = f"{root_dir}/{bms_dir_name}"
        print("Entering dir:", bms_dir_path)
        if not os.path.isdir(bms_dir_path):
            continue
        if not transfer_audio_by_format_in_dir(
            bms_dir_path, *transfer_mode, remove_origin_file=remove_origin_file
        ):
            print("Error occured!")
            break


if __name__ == "__main__":
    main(None)
