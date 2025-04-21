import os
import shutil
from typing import List, Optional, Tuple

from bms import BMSInfo, get_dir_bms_info
from bms_fs import get_bms_folder_dir, get_vaild_fs_name


def set_folder_artist_name(root_dir: str):
    dir_names: List[str] = [
        dir_name
        for dir_name in os.listdir(root_dir)
        if os.path.isdir(os.path.join(root_dir, dir_name))
    ]

    pairs: List[Tuple[str, str]] = []

    for dir_name in dir_names:
        dir_path = os.path.join(root_dir, dir_name)
        if not os.path.isdir(dir_path):
            continue
        # Has been set?
        if dir_name.endswith("]"):
            continue
        bms_info: Optional[BMSInfo] = get_dir_bms_info(dir_path)
        if bms_info is None:
            print(f"Dir {dir_path} has no bms files!")
            continue
        new_dir_name = f"{dir_name} [{get_vaild_fs_name(bms_info.artist)}]"
        print("- Ready to rename: {} -> {}".format(dir_name, new_dir_name))
        pairs.append((dir_name, new_dir_name))

    selection = input("Do transfering? [y/N]:")
    if not selection.lower().startswith("y"):
        print("Aborted.")
        return

    for from_dir_name, target_dir_name in pairs:
        from_dir_path = os.path.join(root_dir, from_dir_name)
        target_dir_path = os.path.join(root_dir, target_dir_name)
        shutil.move(from_dir_path, target_dir_path)


def main(
    root_dir: str = "",
):
    print("该脚本适用于希望在文件夹名后添加“ [艺术家]”的情况。")
    if len(root_dir) == 0:
        root_dir = get_bms_folder_dir()
    set_folder_artist_name(root_dir)


if __name__ == "__main__":
    main()
