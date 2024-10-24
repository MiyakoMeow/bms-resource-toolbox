import os
from typing import List, Tuple

from bms_fs import get_bms_folder_dir
from bms_media.video import (
    VIDEO_PRESET_MPEG1VIDEO_480P,
    VIDEO_PRESET_MPEG1VIDEO_512X512,
    VIDEO_PRESET_WMV1_480P,
    VIDEO_PRESET_WMV1_512X512,
    VideoPreset,
    process_video_in_dir,
)


PRESETS: List[Tuple[str, VideoPreset]] = [
    ("MP4/AVI -> MPEG1VIDEO 480p", VIDEO_PRESET_MPEG1VIDEO_480P),
    ("MP4/AVI -> MPEG1VIDEO 512x512", VIDEO_PRESET_MPEG1VIDEO_512X512),
    ("MP4/AVI -> WMV1 480p", VIDEO_PRESET_WMV1_480P),
    ("MP4/AVI -> WMV1 512x512", VIDEO_PRESET_WMV1_512X512),
]


def main(
    input_exts: List[str] = ["mp4", "avi"],
    presets: List[VideoPreset] = [],
    remove_origin_file: bool = True,
    use_prefered: bool = False,
):
    root_dir = get_bms_folder_dir()

    if len(presets) == 0:
        for i, (mode_str, mode_args) in enumerate(PRESETS):
            print(f"- {i}: {mode_str} ({mode_args})")
        selection_str = input(
            "Select Modes (Type numbers above, split with space, then will exec in order):"
        )
        selections = selection_str.split()
        for selection in selections:
            presets.append(PRESETS[int(selection)][1])

    for bms_dir_name in os.listdir(root_dir):
        bms_dir_path = f"{root_dir}/{bms_dir_name}"
        if not os.path.isdir(bms_dir_path):
            continue
        print("Entering dir:", bms_dir_path)

        is_success = process_video_in_dir(
            bms_dir_path,
            presets=presets,
            remove_origin_file=remove_origin_file,
            use_prefered=use_prefered,
        )
        if not is_success:
            print("Error occured!")
            break


if __name__ == "__main__":
    main()
