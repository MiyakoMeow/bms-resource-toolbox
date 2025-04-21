from concurrent.futures import ThreadPoolExecutor, as_completed
import os
import shutil
import difflib
from typing import List, Optional, Tuple
from dataclasses import dataclass

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


@dataclass
class MoveOptions:
    print_info: bool = False
    replace: bool = True
    replace_skip_unique_file: bool = False
    replace_save_both_unique_file: bool = False


def move_elements_across_dir(
    dir_path_ori: str, dir_path_dst: str, options: MoveOptions = MoveOptions()
):
    if not os.path.isdir(dir_path_ori):
        return
    if not os.path.isdir(dir_path_dst):
        os.mkdir(dir_path_dst)

    next_folder_paths: List[Tuple[str, str]] = []

    def is_same_content(file_a: str, file_b: str) -> bool:
        if not os.path.isfile(file_a):
            return False
        if not os.path.isfile(file_b):
            return False
        with open(file_a, "rb") as fa:
            with open(file_b, "rb") as fb:
                ca = fa.read()
                cb = fb.read()
                return ca == cb

    def move_action(ori_path: str, dst_path: str):
        if options.print_info:
            print(f" - Moving from {ori_path} to {dst_path}")
        # Move
        if os.path.isfile(ori_path):
            move_file(ori_path, dst_path)
        elif os.path.isdir(ori_path):
            move_dir(ori_path, dst_path)

    def move_file(ori_path: str, dst_path: str):
        # Replace?
        if options.replace:
            if not options.replace_skip_unique_file:
                # 不检查内容是否相同？
                shutil.move(ori_path, dst_path)
            elif is_same_content(ori_path, dst_path):
                # 内容相同？
                shutil.move(ori_path, dst_path)
            elif options.replace_save_both_unique_file:
                # 移动并重命名
                file_name = os.path.split(dst_path)[1]
                for i in range(100):
                    name, ext = os.path.splitext(file_name)
                    new_file_name = f"{name}.{i}.{ext}"
                    new_dst_path = os.path.join(dir_path_dst, new_file_name)
                    if os.path.isfile(new_dst_path):
                        continue
                    shutil.move(ori_path, new_dst_path)
                    break
        # Not exists? Move
        elif not os.path.isfile(dst_path):
            shutil.move(ori_path, dst_path)

    def move_dir(ori_path: str, dst_path: str):
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
        move_elements_across_dir(ori_path, dst_path, options)

    # Clean Source
    if options.replace or not is_dir_having_file(dir_path_ori):
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


def dir_similarity(dir_path_a: str, dir_path_b: str) -> float:
    # 相似度
    dir_str_a = " ".join(os.listdir(dir_path_a))
    dir_str_b = " ".join(os.listdir(dir_path_b))
    similarity = difflib.SequenceMatcher(None, dir_str_a, dir_str_b).ratio()
    return similarity
