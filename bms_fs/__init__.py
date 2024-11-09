import os
import shutil
from typing import Optional

from bms import BMSInfo


"""
DIR
"""

_BMS_FOLDER: Optional[str] = None
_BMS_PACK_DIR: Optional[str] = None


def get_bms_folder_dir() -> str:
    global _BMS_FOLDER
    if _BMS_FOLDER is not None:
        return _BMS_FOLDER
    BMS_FOLDER = os.environ.get("BMS_FOLDER")
    if BMS_FOLDER is None:
        BMS_FOLDER = os.path.abspath(".")
    print("Set default dir by env BMS_FOLDER")
    root_dir = input(f"Input root dir path of bms dirs (Default: {BMS_FOLDER}):")
    if len(root_dir.strip()) == 0:
        root_dir = BMS_FOLDER
    _BMS_FOLDER = root_dir
    return _BMS_FOLDER


def get_bms_pack_dir() -> str:
    global _BMS_PACK_DIR
    if _BMS_PACK_DIR is not None:
        return _BMS_PACK_DIR
    BMS_PACK_DIR = os.environ.get("BMS_PACK_DIR")
    if BMS_PACK_DIR is None:
        BMS_PACK_DIR = os.path.abspath(".")
    print("Set default pack dir by env BMS_PACK_DIR")
    root_dir = input(f"Input dir path of bms packs (Default: {BMS_PACK_DIR}):")
    if len(root_dir.strip()) == 0:
        root_dir = BMS_PACK_DIR
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
    for element_name in os.listdir(dir_path_ori):
        ori_path = f"{dir_path_ori}/{element_name}"
        dst_path = f"{dir_path_dst}/{element_name}"
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
            # Make new dir in dst
            if not os.path.isdir(dst_path):
                os.mkdir(dst_path)
            # Child dir
            move_elements_across_dir(ori_path, dst_path, print_info, replace)

    # Clean Source
    if replace or not is_dir_having_file(dir_path_ori):
        for e in os.listdir(dir_path_ori):
            shutil.rmtree(os.path.join(dir_path_ori, e))


def is_dir_having_file(dir_path: str) -> bool:
    has_file = False
    for element_name in os.listdir(dir_path):
        element_path = os.path.join(dir_path, element_name)
        if os.path.isfile(element_path):
            has_file = True
        elif os.path.isdir(element_path):
            has_file = has_file or is_dir_having_file(element_path)

        if has_file:
            break

    return has_file
