import os
import os.path
import shutil
from typing import List, Optional

from bms import BMSInfo, get_dir_bms_info_list
from bms_fs import extract_work_name, get_bms_folder_dir, get_vaild_fs_name


def _pick_bms_in_dir(bms_dir_path: str) -> Optional[BMSInfo]:
    # Find bmses
    bms_list: List[BMSInfo] = get_dir_bms_info_list(bms_dir_path)
    if len(bms_list) == 0:
        return None
    # Split title
    title = extract_work_name([bms.title for bms in bms_list])
    artist = extract_work_name(
        [bms.artist for bms in bms_list], remove_tailing_slash=True
    )
    genre = extract_work_name([bms.genre for bms in bms_list])
    return BMSInfo(title, artist, genre)


def set_dir_name_by_bms(bms_dir_path: str) -> bool:
    info: Optional[BMSInfo] = _pick_bms_in_dir(bms_dir_path)
    if info is None:
        print(f"{bms_dir_path} has no bms/bmson files!")
        return False

    parent_dir, _ = os.path.split(bms_dir_path)
    if parent_dir is None:
        raise Exception("Parent is None!")

    if len(info.title) == 0 and len(info.artist) == 0:
        print(f"{bms_dir_path}: Info title and artist is EMPTY!")
        return False

    # Rename
    new_dir_path = os.path.join(
        parent_dir,
        f"{get_vaild_fs_name(info.title)} [{get_vaild_fs_name(info.artist)}]",
    )

    # Same? Ignore
    if bms_dir_path == new_dir_path:
        return True

    print(f"{bms_dir_path}: Rename! Title: {info.title}; Artist: {info.artist}")
    if os.path.isdir(new_dir_path):
        print(f" - Directory {new_dir_path} exists!")
        return False

    shutil.move(bms_dir_path, new_dir_path)
    return True


def main(root_dir: str):
    """
    该脚本用于重命名作品文件夹。
    格式：“标题 [艺术家]”
    """
    fail_list = []
    for dir_name in os.listdir(root_dir):
        dir_path = os.path.join(root_dir, dir_name)
        if not os.path.isdir(dir_path):
            continue
        result = set_dir_name_by_bms(dir_path)
        if not result:
            fail_list.append(dir_name)
    if len(fail_list) > 0:
        print("Fail Count:", len(fail_list))
        print(fail_list)


if __name__ == "__main__":
    root_dir = get_bms_folder_dir()
    main(root_dir)
