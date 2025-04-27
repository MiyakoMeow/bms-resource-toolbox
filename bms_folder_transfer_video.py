import os
from typing import List, Tuple

from fs import get_bms_folder_dir
from bms_media.video import (
    VIDEO_PRESET_MPEG1VIDEO_480P,
    VIDEO_PRESET_MPEG1VIDEO_512X512,
    VIDEO_PRESET_AVI_480P,
    VIDEO_PRESET_AVI_512X512,
    VIDEO_PRESET_WMV2_480P,
    VIDEO_PRESET_WMV2_512X512,
    VideoPreset,
    process_video_in_dir,
)


PRESETS: List[Tuple[str, VideoPreset]] = [
    ("MP4 -> AVI 512x512", VIDEO_PRESET_AVI_512X512),
    ("MP4 -> AVI 480p", VIDEO_PRESET_AVI_480P),
    ("MP4 -> WMV2 512x512", VIDEO_PRESET_WMV2_512X512),
    ("MP4 -> WMV2 480p", VIDEO_PRESET_WMV2_480P),
    ("MP4 -> MPEG1VIDEO 512x512", VIDEO_PRESET_MPEG1VIDEO_512X512),
    ("MP4 -> MPEG1VIDEO 480p", VIDEO_PRESET_MPEG1VIDEO_480P),
]


def main(
    root_dir: str = "",
    input_exts: List[str] = ["mp4"],
    presets: List[VideoPreset] = [],
    remove_origin_file: bool = True,
    remove_existing_target_file: bool = True,
    use_prefered: bool = False,
):
    if len(root_dir) == 0:
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
        bms_dir_path = os.path.join(root_dir, bms_dir_name)
        if not os.path.isdir(bms_dir_path):
            continue

        is_success = process_video_in_dir(
            bms_dir_path,
            input_exts=input_exts,
            presets=presets,
            remove_origin_file=remove_origin_file,
            remove_existing_target_file=remove_existing_target_file,
            use_prefered=use_prefered,
        )
        if not is_success:
            print("Error occured!")
            break


if __name__ == "__main__":
    main()
