import os
import sys
from typing import List, Tuple

# 获取当前正在执行的Python脚本的绝对路径
current_file_path = os.path.abspath(__file__)

# 获取该脚本所在的目录
current_directory = os.path.dirname(current_file_path)

# 为各模块添加模块寻找路径
sys.path.append(f"{current_directory}/..")

from bms_fs import move_elements_across_dir


def main(root_dir: str):
    if not os.path.isdir(root_dir):
        os.mkdir(root_dir)
    root_folder_name = os.path.split(root_dir)[-1]
    parent_dir = os.path.join(root_dir, "..")
    pairs: List[Tuple[str, str]] = []
    for folder_name in os.listdir(parent_dir):
        folder_path = os.path.join(parent_dir, folder_name)
        if folder_name.startswith(f"{root_folder_name} [") and folder_name.endswith(
            "]"
        ):
            print(f" - {root_dir} <- {folder_path}")
            pairs.append((folder_path, root_dir))

    confirm = input("Confirm? [y/N]")
    if not confirm.lower().startswith("y"):
        return

    for from_dir, to_dir in pairs:
        move_elements_across_dir(from_dir, to_dir)


if __name__ == "__main__":
    root_dir = input("Input the target dir to merge:")
    main(root_dir)
