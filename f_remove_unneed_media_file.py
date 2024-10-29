import os
from typing import Dict, List, Tuple

from bms_fs import get_bms_folder_dir


def remove_unneed_media_files(
    bms_dir_path: str, rules: List[Tuple[List[str], List[str]]]
):
    print(f"Scaning: {bms_dir_path}")
    for file_name in os.listdir(bms_dir_path):
        file_path = os.path.join(bms_dir_path, file_name)
        if not os.path.isfile(file_path):
            continue

        file_ext = file_name.rsplit(".")[-1]
        for upper_exts, lower_exts in rules:
            if file_ext not in upper_exts:
                continue
            # File is empty?
            if os.path.getsize(file_path) == 0:
                print(f" - !x!: File {file_path} is Empty! Skipping...")
                continue
            # File is in upper_exts, search for file in lower_exts.
            for lower_ext in lower_exts:
                replacing_file_path = f"{file_path[:-len(file_ext)] + lower_ext}"
                # File not exist?
                if not os.path.isfile(replacing_file_path):
                    continue
                # Remove file
                print(
                    f"- Remove file {replacing_file_path}, because {file_path} exists."
                )
                os.remove(replacing_file_path)

    # Finished: Count Ext
    ext_count: Dict[str, List[str]] = dict()
    for file_name in os.listdir(bms_dir_path):
        file_path = os.path.join(bms_dir_path, file_name)
        if not os.path.isfile(file_path):
            continue

        # Count ext
        file_ext = file_name.rsplit(".")[-1]
        if ext_count.get(file_ext) is None:
            ext_count.update({file_ext: [file_name]})
        else:
            ext_count[file_ext].append(file_name)

    # Do With Ext Count
    mp4_count = ext_count.get("mp4")
    if mp4_count is not None and len(mp4_count) > 1:
        print(f" - Tips: {bms_dir_path} has more than 1 mp4 files! {mp4_count}")


PRESET_NORMAL: List[Tuple[List[str], List[str]]] = [
    (["mp4"], ["avi", "wmv", "mpg", "mpeg"]),
    (["avi"], ["wmv", "mpg", "mpeg"]),
    (["flac", "wav"], ["ogg"]),
    (["flac"], ["wav"]),
    (["mpg"], ["wmv"]),
]
PRESET_UPDATE_FIRST: List[Tuple[List[str], List[str]]] = [
    (["wav"], ["flac"]),
]
PRESET_MPG_WMV: List[Tuple[List[str], List[str]]] = [
    (["mpg"], ["wmv"]),
]

PRESETS: List[List[Tuple[List[str], List[str]]]] = [
    PRESET_NORMAL,
    PRESET_UPDATE_FIRST,
    PRESET_MPG_WMV,
]


def main(root_dir: str = "", preset: List[Tuple[List[str], List[str]]] = []):
    if len(root_dir) == 0:
        root_dir = get_bms_folder_dir()
    # Select Preset
    if len(preset) == 0:
        for i, preset in enumerate(PRESETS):
            print(f"- {i}: {PRESETS[i]}")
        selection_str = input("Select Preset (Default: 0):")
        selection = 0
        if len(selection_str) > 0:
            selection = int(selection_str)
        preset = PRESETS[selection]
    print(f"Selected: {preset}")

    # Do
    for bms_dir_name in os.listdir(root_dir):
        bms_dir_path = os.path.join(root_dir, bms_dir_name)
        if not os.path.isdir(bms_dir_path):
            continue
        remove_unneed_media_files(
            bms_dir_path,
            preset,
        )


if __name__ == "__main__":
    main()
