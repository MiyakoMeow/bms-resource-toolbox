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


def move_files_across_dir(
    dir_path_ori: str, dir_path_dst: str, print_info: bool = False
):
    for file_name in os.listdir(dir_path_ori):
        ori_path = f"{dir_path_ori}/{file_name}"
        dst_path = f"{dir_path_dst}/{file_name}"
        if print_info:
            print(f" - Moving from {ori_path} to {dst_path}")
        shutil.move(ori_path, dst_path)
