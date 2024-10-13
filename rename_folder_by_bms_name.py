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

    # Rename
    print(f"{dir_path} found bms title: {info.title} artist: {info.artist}")
    new_dir_path = f"{dir_path}. {get_vaild_fs_name(info.title)} [{get_vaild_fs_name(info.artist)}]"
    shutil.move(dir_path, new_dir_path)

    # Move out files
    dir_path = new_dir_path
    dir_inner_list = os.listdir(dir_path)
    if len(dir_inner_list) > 1:
        return
    dir_inner_path = f"{dir_path}/{dir_inner_list[0]}"
    print(f"Moving files in {dir_inner_path} to parent folder")
    move_files_across_dir(dir_inner_path, dir_path)
    os.rmdir(dir_inner_path)


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
