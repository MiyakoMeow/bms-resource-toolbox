import os
from typing import List
import difflib

from bms_fs import get_bms_folder_dir


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


def main(
    root_dir: str = "",
):
    if len(root_dir) == 0:
        root_dir = get_bms_folder_dir()
    scan_folder_similar_folders(root_dir)


if __name__ == "__main__":
    main()
