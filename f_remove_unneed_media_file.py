import os
from typing import Dict, List, Tuple

from bms_fs import get_bms_folder_dir


def remove_unneed_media_files(
    bms_dir_path: str, rules: List[Tuple[List[str], List[str]]]
):
    print(f"Scaning: {bms_dir_path}")
    for file_name in os.listdir(bms_dir_path):
        file_path = f"{bms_dir_path}/{file_name}"
        if not os.path.isfile(file_path):
            continue

        file_ext = file_name.rsplit(".")[-1]
        for upper_exts, lower_exts in rules:
            if file_ext not in upper_exts:
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
        file_path = f"{bms_dir_path}/{file_name}"
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


if __name__ == "__main__":
    root_dir = get_bms_folder_dir()
    for bms_dir_name in os.listdir(root_dir):
        bms_dir_path = f"{root_dir}/{bms_dir_name}"
        if not os.path.isdir(bms_dir_path):
            continue
        remove_unneed_media_files(
            bms_dir_path,
            [
                (["mp4", "avi"], ["wmv", "mpg", "mpeg"]),
                (["flac", "wav"], ["ogg"]),
                (["flac"], ["wav"]),
            ],
        )
