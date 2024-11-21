import os
import os.path
import shutil
from typing import List, Optional

from bms import BMSDifficulty, BMSInfo, get_dir_bms_info_list
from bms_fs import get_bms_folder_dir, get_vaild_fs_name


def _get_bms(bms_dir_path: str) -> Optional[BMSInfo]:
    # Find bmses
    bms_list: List[BMSInfo] = get_dir_bms_info_list(bms_dir_path)
    # Find suitable level 1
    bms_list_lv1 = [
        bms
        for bms in bms_list
        if bms.difficulty != BMSDifficulty.Insane and 1 <= bms.playlevel <= 12
    ]
    if len(bms_list_lv1) > 0:
        bms = bms_list_lv1[0]
        return bms
    elif len(bms_list) > 0:
        bms = bms_list[0]
        return bms
    else:
        return None


def set_dir_name_by_bms(bms_dir_path: str):
    if not os.path.split(bms_dir_path)[-1].strip().isdigit():
        # print(f"{dir_path} has been renamed! Skipping...")
        return

    info: Optional[BMSInfo] = _get_bms(bms_dir_path)
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
    for dir_name in os.listdir(root_dir):
        dir_path = f"{root_dir}/{dir_name}"
        if not os.path.isdir(dir_path):
            continue
        set_dir_name_by_bms(dir_path)


if __name__ == "__main__":
    root_dir = get_bms_folder_dir()
    main(root_dir)
