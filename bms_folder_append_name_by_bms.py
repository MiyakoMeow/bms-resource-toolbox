import os
import os.path
import shutil
from typing import List, Optional

from bms import BMSDifficulty, BMSInfo, get_dir_bms_info_list
from bms_fs import get_bms_folder_dir, get_vaild_fs_name


def _pick_bms_in_dir(bms_dir_path: str) -> Optional[BMSInfo]:
    # Find bmses
    bms_list: List[BMSInfo] = get_dir_bms_info_list(bms_dir_path)
    # Find Beginner
    bms_list_filtered = [
        bms
        for bms in bms_list
        if bms.difficulty == BMSDifficulty.Beginner and 1 <= bms.playlevel <= 6
    ]
    if len(bms_list_filtered) > 0:
        bms = bms_list_filtered[0]
        return bms
    # Find Normal
    bms_list_filtered = [
        bms
        for bms in bms_list
        if bms.difficulty == BMSDifficulty.Normal and 4 <= bms.playlevel <= 9
    ]
    if len(bms_list_filtered) > 0:
        bms = bms_list_filtered[0]
        return bms
    # Find Hyper
    bms_list_filtered = [
        bms
        for bms in bms_list
        if bms.difficulty == BMSDifficulty.Hyper and 7 <= bms.playlevel <= 11
    ]
    if len(bms_list_filtered) > 0:
        bms = bms_list_filtered[0]
        return bms
    # Last: Pick the first
    if len(bms_list) > 0:
        bms = bms_list[0]
        return bms
    return None


def set_dir_name_by_bms(bms_dir_path: str):
    if not os.path.split(bms_dir_path)[-1].strip().isdigit():
        # print(f"{dir_path} has been renamed! Skipping...")
        return

    info: Optional[BMSInfo] = _pick_bms_in_dir(bms_dir_path)
    if info is None:
        print(f"{bms_dir_path} has no bms/bmson files!")
        return

    # Deal with info
    print(f"{bms_dir_path} found bms title: {info.title} artist: {info.artist}")
    title = info.title
    artist = info.artist

    # Rename
    new_dir_path = (
        f"{bms_dir_path}. {get_vaild_fs_name(title)} [{get_vaild_fs_name(artist)}]"
    )
    shutil.move(bms_dir_path, new_dir_path)


def main(root_dir: str):
    print("该脚本适用于原有文件夹名与BMS文件无关内容的情况。")
    print("会在文件夹名后添加“. 标题 [艺术家]”")
    for dir_name in os.listdir(root_dir):
        dir_path = f"{root_dir}/{dir_name}"
        if not os.path.isdir(dir_path):
            continue
        set_dir_name_by_bms(dir_path)


if __name__ == "__main__":
    root_dir = get_bms_folder_dir()
    main(root_dir)
