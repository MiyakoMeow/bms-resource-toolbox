import os
import shutil
from typing import Callable, List, Tuple
import sys

# 获取当前正在执行的Python脚本的绝对路径
current_file_path = os.path.abspath(__file__)

# 获取该脚本所在的目录
current_directory = os.path.dirname(current_file_path)

# 为各模块添加模块寻找路径
sys.path.append(f"{current_directory}/..")

from bms import BMSDifficulty, BMSInfo, get_dir_bms_info_list
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
        # Find bmses
        bms_list: List[BMSInfo] = get_dir_bms_info_list(dir_path)

        # Add pair
        def add_pair(bms: BMSInfo):
            new_dir_name = f"{dir_name} [{get_vaild_fs_name(bms.artist)}]"
            print("- Ready to rename: {} -> {}".format(dir_name, new_dir_name))
            pairs.append((dir_name, new_dir_name))

        # Find suitable whitestar level
        # Filter 1: hyperless
        def bms_fliter_hyperless(bms: BMSInfo) -> bool:
            return (
                bms.difficulty != BMSDifficulty.Insane
                and bms.difficulty != BMSDifficulty.Unknown
                and bms.difficulty != BMSDifficulty.Another
                and 1 <= bms.playlevel <= 10
            )

        # Filter 2: whitestar
        def bms_fliter_whitestar(bms: BMSInfo) -> bool:
            return (
                bms.difficulty != BMSDifficulty.Insane
                and bms.difficulty != BMSDifficulty.Unknown
                and 8 <= bms.playlevel <= 12
            )

        bms_list_hyperless = [bms for bms in bms_list if bms_fliter_hyperless(bms)]
        if len(bms_list_hyperless) > 0:
            bms = bms_list_hyperless[0]
            add_pair(bms)
            continue

        bms_list_whitestar = [bms for bms in bms_list if bms_fliter_whitestar(bms)]
        if len(bms_list_whitestar) > 0:
            bms = bms_list_whitestar[0]
            add_pair(bms)
            continue

        # Filter end: first
        if len(bms_list) > 0:
            bms = bms_list[0]
            add_pair(bms)
            continue

        print(f"Dir {dir_path} has no bms files!")

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
    if len(root_dir) == 0:
        root_dir = get_bms_folder_dir()
    set_folder_artist_name(root_dir)


if __name__ == "__main__":
    main()
