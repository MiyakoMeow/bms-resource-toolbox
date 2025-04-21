import os
import os.path
import shutil
from typing import Optional

from bms import BMSInfo, get_dir_bms_info
from bms_fs import (
    ReplaceAction,
    ReplaceOptions,
    bms_dir_similarity,
    get_vaild_fs_name,
    move_elements_across_dir,
    get_bms_folder_dir,
)


def set_dir_name_by_bms(bms_dir_path: str) -> bool:
    info: Optional[BMSInfo] = get_dir_bms_info(bms_dir_path)
    while info is None:
        print(f"{bms_dir_path} has no bms/bmson files! Trying to move out.")
        bms_dir_elements = os.listdir(bms_dir_path)
        if len(bms_dir_elements) == 0:
            print(" - Empty dir! Deleting...")
            try:
                os.rmdir(bms_dir_path)
            except PermissionError as e:
                print(e)
            return False
        if len(bms_dir_elements) != 1:
            print(f" - Element count: {len(bms_dir_elements)}")
            return False
        bms_dir_inner = os.path.join(bms_dir_path, bms_dir_elements[0])
        if not os.path.isdir(bms_dir_inner):
            print(f" - Folder has only a file: {bms_dir_elements[0]}")
            return False
        print(" - Moving out files...")
        move_elements_across_dir(bms_dir_inner, bms_dir_path)
        info = get_dir_bms_info(bms_dir_path)

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
    if not os.path.isdir(new_dir_path):
        # Move Directly
        shutil.move(bms_dir_path, new_dir_path)
        return True

    # Same dir?
    similarity = bms_dir_similarity(bms_dir_path, new_dir_path)
    print(f" - Directory {new_dir_path} exists! Similarity: {similarity}")
    if similarity < 0.9:
        print("- Merge canceled.")
        return False

    print(" - Merge start!")
    move_elements_across_dir(
        bms_dir_path,
        new_dir_path,
        replace_options=ReplaceOptions(
            ext=dict(
                (ext, ReplaceAction.CheckReplace)
                for ext in [
                    "bms",
                    "bml",
                    "bme",
                    "pms",
                    "txt",
                    "bmson",
                ]
            ),
            default=ReplaceAction.Replace,
        ),
    )
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
