from concurrent.futures import ThreadPoolExecutor, as_completed
import os
import shutil
from typing import List, Optional, Tuple

from bms import BMSInfo


"""
DIR
"""

_BMS_FOLDER: Optional[str] = None
_BMS_PACK_DIR: Optional[str] = None


def get_bms_folder_dir(tips: bool = True, use_default: bool = True) -> str:
    global _BMS_FOLDER
    if _BMS_FOLDER is not None:
        return _BMS_FOLDER
    BMS_FOLDER = os.environ.get("BMS_FOLDER")
    if BMS_FOLDER is None:
        BMS_FOLDER = os.path.abspath(".")
    if tips:
        print("Set default dir by env BMS_FOLDER")
        print(f"Input root dir path of bms dirs (Default: {BMS_FOLDER}):", end="")
    root_dir = input()
    if len(root_dir.strip()) == 0:
        if use_default:
            root_dir = BMS_FOLDER
        else:
            raise Exception("Default Value Disabled.")
    _BMS_FOLDER = root_dir
    return _BMS_FOLDER


def get_bms_pack_dir(tips: bool = True, use_default: bool = True) -> str:
    global _BMS_PACK_DIR
    if _BMS_PACK_DIR is not None:
        return _BMS_PACK_DIR
    BMS_PACK_DIR = os.environ.get("BMS_PACK_DIR")
    if BMS_PACK_DIR is None:
        BMS_PACK_DIR = os.path.abspath(".")
    if tips:
        print("Set default pack dir by env BMS_PACK_DIR")
        print(f"Input dir path of bms packs (Default: {BMS_PACK_DIR}):", end="")
    root_dir = input()
    if len(root_dir.strip()) == 0:
        if use_default:
            root_dir = BMS_PACK_DIR
        else:
            raise Exception("Default Value Disabled.")
    _BMS_PACK_DIR = root_dir
    return _BMS_PACK_DIR


"""
FS
"""


def get_vaild_fs_name(ori_name: str) -> str:
    """
    Signs:
    ：＼／＊？＂＜＞｜
    """
    return (
        ori_name.replace(":", "：")
        .replace("\\", "＼")
        .replace("/", "／")
        .replace("*", "＊")
        .replace("?", "？")
        .replace("!", "！")
        .replace('"', "＂")
        .replace("<", "＜")
        .replace(">", "＞")
        .replace("|", "｜")
    )


def get_folder_name(id: str, info: BMSInfo) -> str:
    return f"{id}. {get_vaild_fs_name(info.title)} [{get_vaild_fs_name(info.artist)}]"


def move_elements_across_dir(
    dir_path_ori: str,
    dir_path_dst: str,
    print_info: bool = False,
    replace: bool = True,
):
    if not os.path.isdir(dir_path_ori):
        return
    if not os.path.isdir(dir_path_dst):
        os.mkdir(dir_path_dst)

    next_folder_paths: List[Tuple[str, str]] = []

    def move_action(ori_path: str, dst_path: str):
        if print_info:
            print(f" - Moving from {ori_path} to {dst_path}")
        # Move
        if os.path.isfile(ori_path):
            # Replace?
            if os.path.isfile(dst_path) and replace:
                os.remove(dst_path)
            # Move
            if not os.path.isfile(dst_path):
                shutil.move(ori_path, dst_path)
        elif os.path.isdir(ori_path):
            # Directly move dir
            if not os.path.isdir(dst_path):
                shutil.move(ori_path, dst_path)
            else:
                # Add child dir
                next_folder_paths.append((ori_path, dst_path))

    with ThreadPoolExecutor(max_workers=4) as executor:
        # 提交任务
        dir_lists: List[Tuple[str, str]] = [
            (
                os.path.join(dir_path_ori, element_name),
                os.path.join(dir_path_dst, element_name),
            )
            for element_name in os.listdir(dir_path_ori)
        ]
        futures = [
            executor.submit(
                move_action,
                path_ori,
                path_dst,
            )
            for path_ori, path_dst in dir_lists
        ]
        # 等待任务完成
        for _ in as_completed(futures):
            pass

    # Next Level
    for ori_path, dst_path in next_folder_paths:
        move_elements_across_dir(ori_path, dst_path, print_info, replace)

    # Clean Source
    if replace or not is_dir_having_file(dir_path_ori):
        try:
            shutil.rmtree(dir_path_ori)
        except PermissionError:
            print(f" x PermissionError! ({dir_path_ori})")


def is_dir_having_file(dir_path: str) -> bool:
    has_file = False
    for element_name in os.listdir(dir_path):
        element_path = os.path.join(dir_path, element_name)
        if os.path.isfile(element_path) and os.path.getsize(element_path) > 0:
            has_file = True
        elif os.path.isdir(element_path):
            has_file = has_file or is_dir_having_file(element_path)

        if has_file:
            break

    return has_file
