import os
import shutil
import sys

# 获取当前正在执行的Python脚本的绝对路径
current_file_path = os.path.abspath(__file__)

# 获取该脚本所在的目录
current_directory = os.path.dirname(current_file_path)

# 为各模块添加模块寻找路径
sys.path.append(f"{current_directory}/..")

from bms_fs import get_bms_folder_dir, is_dir_having_file


def main(bms_dir: str = ""):
    if len(bms_dir) == 0:
        bms_dir = get_bms_folder_dir()

    for dir_name in os.listdir(bms_dir):
        dir_path = os.path.join(bms_dir, dir_name)
        if not os.path.isdir(dir_path):
            continue
        if not is_dir_having_file(dir_path):
            try:
                print(f"Remove empty dir: {dir_path}")
                shutil.rmtree(dir_path)
            except PermissionError:
                print(" x PermissionError!")


if __name__ == "__main__":
    main()
