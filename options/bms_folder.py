import difflib
import os
import shutil
from typing import List, Optional, Tuple

from bms import BMSInfo, get_dir_bms_info
from fs import bms_dir_similarity
from fs.name import get_vaild_fs_name
from fs.move import REPLACE_OPTION_UPDATE_PACK, move_elements_across_dir


def append_artist_name_by_bms(root_dir: str):
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


def _workdir_append_name_by_bms(work_dir: str) -> bool:
    """
    该脚本适用于原有文件夹名与BMS文件无关内容的情况。
    会在文件夹名后添加“. 标题 [艺术家]”
    """
    if not os.path.split(work_dir)[-1].strip().isdigit():
        print(f"{work_dir} has been renamed! Skipping...")
        return False

    info: Optional[BMSInfo] = get_dir_bms_info(work_dir)
    if info is None:
        print(f"{work_dir} has no bms/bmson files!")
        return False

    # Deal with info
    print(f"{work_dir} found bms title: {info.title} artist: {info.artist}")
    title = info.title
    artist = info.artist

    # Rename
    new_dir_path = (
        f"{work_dir}. {get_vaild_fs_name(title)} [{get_vaild_fs_name(artist)}]"
    )
    shutil.move(work_dir, new_dir_path)
    return True


def append_name_by_bms(root_dir: str):
    """
    该脚本用于重命名作品文件夹。
    格式：“标题 [艺术家]”
    """
    fail_list = []
    for dir_name in os.listdir(root_dir):
        dir_path = os.path.join(root_dir, dir_name)
        if not os.path.isdir(dir_path):
            continue
        result = _workdir_append_name_by_bms(dir_path)
        if not result:
            fail_list.append(dir_name)
    if len(fail_list) > 0:
        print("Fail Count:", len(fail_list))
        print(fail_list)


def _set_workdir_name_by_bms(work_dir: str) -> bool:
    info: Optional[BMSInfo] = get_dir_bms_info(work_dir)
    while info is None:
        print(f"{work_dir} has no bms/bmson files! Trying to move out.")
        bms_dir_elements = os.listdir(work_dir)
        if len(bms_dir_elements) == 0:
            print(" - Empty dir! Deleting...")
            try:
                os.rmdir(work_dir)
            except PermissionError as e:
                print(e)
            return False
        if len(bms_dir_elements) != 1:
            print(f" - Element count: {len(bms_dir_elements)}")
            return False
        bms_dir_inner = os.path.join(work_dir, bms_dir_elements[0])
        if not os.path.isdir(bms_dir_inner):
            print(f" - Folder has only a file: {bms_dir_elements[0]}")
            return False
        print(" - Moving out files...")
        move_elements_across_dir(bms_dir_inner, work_dir)
        info = get_dir_bms_info(work_dir)

    parent_dir, _ = os.path.split(work_dir)
    if parent_dir is None:
        raise Exception("Parent is None!")

    if len(info.title) == 0 and len(info.artist) == 0:
        print(f"{work_dir}: Info title and artist is EMPTY!")
        return False

    # Rename
    new_dir_path = os.path.join(
        parent_dir,
        f"{get_vaild_fs_name(info.title)} [{get_vaild_fs_name(info.artist)}]",
    )

    # Same? Ignore
    if work_dir == new_dir_path:
        return True

    print(f"{work_dir}: Rename! Title: {info.title}; Artist: {info.artist}")
    if not os.path.isdir(new_dir_path):
        # Move Directly
        shutil.move(work_dir, new_dir_path)
        return True

    # Same dir?
    similarity = bms_dir_similarity(work_dir, new_dir_path)
    print(f" - Directory {new_dir_path} exists! Similarity: {similarity}")
    if similarity < 0.8:
        print(" - Merge canceled.")
        return False

    print(" - Merge start!")
    move_elements_across_dir(
        work_dir,
        new_dir_path,
        replace_options=REPLACE_OPTION_UPDATE_PACK,
    )
    return True


def set_name_by_bms(root_dir: str):
    """
    该脚本用于重命名作品文件夹。
    格式：“标题 [艺术家]”
    """
    fail_list = []
    for dir_name in os.listdir(root_dir):
        dir_path = os.path.join(root_dir, dir_name)
        if not os.path.isdir(dir_path):
            continue
        result = _set_workdir_name_by_bms(dir_path)
        if not result:
            fail_list.append(dir_name)
    if len(fail_list) > 0:
        print("Fail Count:", len(fail_list))
        print(fail_list)


def copy_numbered_workdir_names(root_dir_from: str, root_dir_to: str):
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


def undo_set_name(root_dir: str):
    for dir_name in os.listdir(root_dir):
        dir_path = os.path.join(root_dir, dir_name)
        if not os.path.isdir(dir_path):
            continue
        new_dir_name = dir_name.split(" ")[0]
        new_dir_path = os.path.join(root_dir, new_dir_name)
        if dir_name == new_dir_name:
            continue
        print(f"Rename {dir_name} to {new_dir_name}")
        shutil.move(dir_path, new_dir_path)
