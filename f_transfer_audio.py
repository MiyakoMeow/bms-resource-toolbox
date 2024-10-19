import os
from typing import Tuple, List

from bms_fs import get_bms_folder_dir
from bms_media import (
    AUDIO_PRESET_FLAC,
    AUDIO_PRESET_OGG,
    AUDIO_PRESET_OGG_320K,
    AUDIO_PRESET_WAV,
    AudioPreset,
    transfer_audio_by_format_in_dir,
)


MODES: List[Tuple[str, List[str], List[AudioPreset]]] = [
    ("For HQ: WAV to FLAC", ["wav"], [AUDIO_PRESET_FLAC]),
    (
        "For LQ: FLAC/WAV to OGG 320k/default",
        ["wav", "flac"],
        [AUDIO_PRESET_OGG_320K, AUDIO_PRESET_OGG],
    ),
    ("Compress: FLAC to OGG 320k", ["flac"], [AUDIO_PRESET_OGG_320K]),
    ("Compress: FLAC to OGG", ["flac"], [AUDIO_PRESET_OGG_320K]),
    ("Reverse: FLAC to WAV", ["flac"], [AUDIO_PRESET_WAV]),
    ("Reverse: OGG to WAV", ["ogg"], [AUDIO_PRESET_WAV]),
]


def main(
    input_ext: List[str] = [],
    transfer_mode: List[AudioPreset] = [],
    remove_origin_file: bool = True,
):
    root_dir = get_bms_folder_dir()

    # Select Modes
    if len(transfer_mode) == 0 or len(input_ext) == 0:
        for i, (mode_str, mode_input_exts, mode_presets) in enumerate(MODES):
            print(f"- {i}: {mode_str} ({mode_input_exts}) ({mode_presets})")
        selection = int(input("Select Mode (Type numbers above):"))
        input_ext = MODES[selection][1]
        transfer_mode = MODES[selection][2]

    for bms_dir_name in os.listdir(root_dir):
        bms_dir_path = f"{root_dir}/{bms_dir_name}"
        print(
            "Entering dir:",
            bms_dir_path,
            "Input ext:",
            input_ext,
            "Preset:",
            transfer_mode,
        )
        if not os.path.isdir(bms_dir_path):
            continue
        if not transfer_audio_by_format_in_dir(
            bms_dir_path,
            input_ext,
            transfer_mode,
            remove_origin_file=remove_origin_file,
        ):
            print("Error occured!")
            break


if __name__ == "__main__":
    main()
