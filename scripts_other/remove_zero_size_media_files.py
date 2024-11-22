import os
import sys

# 获取当前正在执行的Python脚本的绝对路径
current_file_path = os.path.abspath(__file__)

# 获取该脚本所在的目录
current_directory = os.path.dirname(current_file_path)

# 为各模块添加模块寻找路径
sys.path.append(f"{current_directory}/..")

from bms_fs import get_bms_folder_dir


def main(parent_dir: str = ""):
    if len(parent_dir) == 0:
        parent_dir = get_bms_folder_dir()

    if not os.path.isdir(parent_dir):
        print("Not a vaild dir! Aborting...")
        pass

    for element_name in os.listdir(parent_dir):
        element_path = os.path.join(parent_dir, element_name)
        if os.path.isfile(element_path):
            if not (
                element_name.endswith(".ogg")
                or element_name.endswith(".wav")
                or element_name.endswith(".flac")
                or element_name.endswith(".bmp")
                or element_name.endswith(".mpg")
                or element_name.endswith(".wmv")
                or element_name.endswith(".mp4")
            ):
                continue
            if os.path.getsize(element_path) > 0:
                continue
            try:
                print(f" - Remove empty file: {element_path}")
                os.remove(element_path)
            except PermissionError:
                print(" x PermissionError!")
        elif os.path.isdir(element_path):
            main(parent_dir=element_path)


if __name__ == "__main__":
    main()
