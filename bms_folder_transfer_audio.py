import os
from typing import Tuple, List

from bms_fs import get_bms_folder_dir
from bms_media.audio import (
    AUDIO_PRESET_FLAC,
    AUDIO_PRESET_FLAC_FFMPEG,
    AUDIO_PRESET_OGG_Q10,
    AUDIO_PRESET_WAV_FFMPEG,
    AUDIO_PRESET_WAV_FROM_FLAC,
    AudioPreset,
    transfer_audio_by_format_in_dir,
)


MODES: List[Tuple[str, List[str], List[AudioPreset]]] = [
    (
        "Convert: WAV to FLAC",
        ["wav"],
        [
            AUDIO_PRESET_FLAC,
            AUDIO_PRESET_FLAC_FFMPEG,
        ],
    ),
    ("Compress: FLAC to OGG Q10", ["flac"], [AUDIO_PRESET_OGG_Q10]),
    ("Compress: WAV to OGG Q10", ["wav"], [AUDIO_PRESET_OGG_Q10]),
    (
        "Reverse: FLAC to WAV",
        ["flac"],
        [AUDIO_PRESET_WAV_FROM_FLAC, AUDIO_PRESET_WAV_FFMPEG],
    ),
]


def main(
    root_dir: str = "",
    input_ext: List[str] = [],
    transfer_mode: List[AudioPreset] = [],
    remove_origin_file_when_success: bool = True,
    remove_origin_file_when_failed: bool = True,
    skip_on_fail: bool = False,
):
    if len(root_dir) == 0:
        root_dir = get_bms_folder_dir()

    # Select Modes
    if len(transfer_mode) == 0 or len(input_ext) == 0:
        for i, (mode_str, mode_input_exts, mode_presets) in enumerate(MODES):
            print(f"- {i}: {mode_str} ({mode_input_exts}) ({mode_presets})")
        selection = int(input("Select Mode (Type numbers above):"))
        input_ext = MODES[selection][1]
        transfer_mode = MODES[selection][2]

    for bms_dir_name in os.listdir(root_dir):
        bms_dir_path = os.path.join(root_dir, bms_dir_name)
        if not os.path.isdir(bms_dir_path):
            continue
        is_success = transfer_audio_by_format_in_dir(
            bms_dir_path,
            input_ext,
            transfer_mode,
            remove_origin_file_when_success=remove_origin_file_when_success,
            remove_origin_file_when_failed=remove_origin_file_when_failed,
        )
        if not is_success:
            print(" - Dir:", bms_dir_path, "Error occured!")
            if skip_on_fail:
                break


if __name__ == "__main__":
    main()
