import os
import os.path
import shutil
from typing import Optional

from bms import get_dir_bms_info, BMSInfo
from bms_fs import get_bms_folder_dir, get_vaild_fs_name


def deal_with_dir(dir_path: str):
    if not os.path.split(dir_path)[-1].strip().isdigit():
        # print(f"{dir_path} has been renamed! Skipping...")
        return
    info: Optional[BMSInfo] = get_dir_bms_info(dir_path)
    if info is None:
        print(f"{dir_path} has no bms/bmson files!")
        return

    # Deal with info
    print(f"{dir_path} found bms title: {info.title} artist: {info.artist}")
    title = info.title
    artist = info.artist

    # Rename
    new_dir_path = (
        f"{dir_path}. {get_vaild_fs_name(title)} [{get_vaild_fs_name(artist)}]"
    )
    shutil.move(dir_path, new_dir_path)


if __name__ == "__main__":
    root_dir = get_bms_folder_dir()
    for dir_name in os.listdir(root_dir):
        dir_path = f"{root_dir}/{dir_name}"
        if not os.path.isdir(dir_path):
            continue
        deal_with_dir(dir_path)
