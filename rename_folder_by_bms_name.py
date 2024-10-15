import os
import os.path
import shutil
from typing import Optional

from bms import get_dir_bms_info, BMSInfo
from bms_fs import get_vaild_fs_name, move_files_across_dir

BOFTT_DIR = os.environ.get("BOFTT_DIR")
if BOFTT_DIR is None:
    BOFTT_DIR = os.path.abspath(".")


def deal_with_dir(dir_path: str):
    if not dir_path.split("/")[-1].split("\\")[-1].strip().isdigit():
        # print(f"{dir_path} has been renamed! Skipping...")
        return
    info: Optional[BMSInfo] = get_dir_bms_info(dir_path)
    if info is None:
        print(f"{dir_path} has no bms/bmson files!")
        return

    # Scan folder
    file_count = 0
    folder_count = 0
    only_folder_name = None
    for inner_element_name in os.listdir(dir_path):
        inner_path = f"{dir_path}/{inner_element_name}"
        if os.path.isfile(inner_path):
            file_count += 1
        elif os.path.isdir(inner_path):
            folder_count += 1
            only_folder_name = inner_element_name

    # Check folder
    if folder_count == 0 and file_count == 0:
        print(f"{dir_path} is empty!")
        return
    if (folder_count == 0 or folder_count == 1) and 1 < file_count <= 10:
        print(
            f"{dir_path} has no enough files, is likely not a bms folder, or not arranged!"
        )
        return
    if folder_count > 1:
        print(f"{dir_path} has extra folders!")
        return

    # Move out files
    if only_folder_name is not None:
        dir_inner_path = f"{dir_path}/{only_folder_name}"
        print(f"Moving files in {dir_inner_path} to parent folder")
        move_files_across_dir(dir_inner_path, dir_path)
        os.rmdir(dir_inner_path)

    # Rename
    print(f"{dir_path} found bms title: {info.title} artist: {info.artist}")
    new_dir_path = f"{dir_path}. {get_vaild_fs_name(info.title)} [{get_vaild_fs_name(info.artist)}]"
    shutil.move(dir_path, new_dir_path)


if __name__ == "__main__":
    print("Set default dir by env BOFTT_DIR")
    root_dir = input(f"Input root dir of bms dirs (Default: {BOFTT_DIR}):")
    if len(root_dir.strip()) == 0:
        root_dir = BOFTT_DIR
    for dir_name in os.listdir(root_dir):
        dir_path = f"{root_dir}/{dir_name}"
        if not os.path.isdir(dir_path):
            continue
        deal_with_dir(dir_path)
