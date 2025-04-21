import os
import os.path
import shutil
from typing import Optional

from bms import BMSInfo, get_dir_bms_info
from bms_fs import get_bms_folder_dir, get_vaild_fs_name


def set_dir_name_by_bms(bms_dir_path: str):
    if not os.path.split(bms_dir_path)[-1].strip().isdigit():
        # print(f"{dir_path} has been renamed! Skipping...")
        return

    info: Optional[BMSInfo] = get_dir_bms_info(bms_dir_path)
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
