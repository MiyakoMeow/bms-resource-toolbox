import os
from typing import Optional

from bms_fs import get_bms_folder_dir
from bms_media import (
    VIDEO_PRESET_WMV_480P,
    VIDEO_PRESET_WMV_512X512,
    process_video_in_dir,
)


PRESETS = [
    ("MP4/AVI -> WMV 512x512", VIDEO_PRESET_WMV_512X512),
    ("MP4/AVI -> WMV 480p", VIDEO_PRESET_WMV_480P),
]


def main(
    preset: Optional[int] = None,
    remove_origin_file: bool = True,
):
    root_dir = get_bms_folder_dir()

    if preset is None:
        for i, (mode_str, mode_args) in enumerate(PRESETS):
            print(f"- {i}: {mode_str} ({mode_args})")
        selection = int(input("Select Mode (Type numbers above):"))
        preset = PRESETS[selection][1]

    for bms_dir_name in os.listdir(root_dir):
        bms_dir_path = f"{root_dir}/{bms_dir_name}"
        print("Entering dir:", bms_dir_path)
        if not os.path.isdir(bms_dir_path):
            continue
        if not process_video_in_dir(
            bms_dir_path, preset, remove_origin_file=remove_origin_file
        ):
            print("Error occured!")
            break


if __name__ == "__main__":
    main()
