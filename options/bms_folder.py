import difflib
import os
import shutil
from typing import List, Optional, Tuple

from bms import BMSInfo, get_dir_bms_info
from fs.name import get_vaild_fs_name


def append_artist_name(root_dir: str):
    """该脚本适用于希望在作品文件夹名后添加“ [艺术家]”的情况。"""
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


def _workdir_append_name_by_bms(work_dir: str):
    """
    该脚本适用于原有文件夹名与BMS文件无关内容的情况。
    会在文件夹名后添加“. 标题 [艺术家]”
    """
    if not os.path.split(work_dir)[-1].strip().isdigit():
        # print(f"{dir_path} has been renamed! Skipping...")
        return

    info: Optional[BMSInfo] = get_dir_bms_info(work_dir)
    if info is None:
        print(f"{work_dir} has no bms/bmson files!")
        return

    # Deal with info
    print(f"{work_dir} found bms title: {info.title} artist: {info.artist}")
    title = info.title
    artist = info.artist

    # Rename
    new_dir_path = (
        f"{work_dir}. {get_vaild_fs_name(title)} [{get_vaild_fs_name(artist)}]"
    )
    shutil.move(work_dir, new_dir_path)


def copy_workdir_names(root_dir_from: str, root_dir_to: str):
    """
    该脚本使用于以下情况：
    已经有一个文件夹A，它的子文件夹名为“”等带有编号+小数点的形式。
    现在有另一个文件夹B，它的子文件夹名都只有编号。
    将A中的子文件夹名，同步给B的对应的子文件夹。
    """
    src_dir_names = [
        dir_name
        for dir_name in os.listdir(root_dir_from)
        if os.path.isdir(os.path.join(root_dir_from, dir_name))
    ]
    # List Dst Dir
    for dir_name in os.listdir(root_dir_to):
        dir_path = os.path.join(root_dir_to, dir_name)
        # Get Num
        dir_num = dir_name.split(" ")[0].split(".")[0]
        if not dir_num.isdigit():
            continue
        # Search src name
        for src_name in src_dir_names:
            if not src_name.startswith(dir_num):
                continue
            # Rename
            target_dir_path = os.path.join(root_dir_to, src_name)
            print(f"Rename {dir_name} to {src_name}")
            shutil.move(dir_path, target_dir_path)
            break


def scan_folder_similar_folders(root_dir: str, similarity_trigger: float = 0.7):
    dir_names: List[str] = [
        dir_name
        for dir_name in os.listdir(root_dir)
        if os.path.isdir(os.path.join(root_dir, dir_name))
    ]
    print(f"当前目录下有{len(dir_names)}个文件夹。")
    # Sort
    dir_names.sort()
    # Scan
    for i, dir_name in enumerate(dir_names):
        if i == 0:
            continue
        former_dir_name = dir_names[i - 1]
        # 相似度
        similarity = difflib.SequenceMatcher(None, former_dir_name, dir_name).ratio()
        if similarity < similarity_trigger:
            continue
        print(f"发现相似项：{former_dir_name} <=> {dir_name}")
