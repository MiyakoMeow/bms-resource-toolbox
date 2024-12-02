import os
from typing import List

from bms_fs import get_bms_folder_dir


def main(parent_dir: str, print_dir: bool = False):
    if print_dir:
        print(f"Entering dir: {parent_dir}")

    if not os.path.isdir(parent_dir):
        print("Not a vaild dir! Aborting...")
        pass

    next_dir_list: List[str] = []

    for element_name in os.listdir(parent_dir):
        element_path = os.path.join(parent_dir, element_name)
        if os.path.isfile(element_path):
            # print(f" - Found file: {element_name}")
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
            # print(f" - Found dir: {element_name}")
            next_dir_list.append(element_name)

    for next_dir_name in next_dir_list:
        main(parent_dir=os.path.join(parent_dir, next_dir_name), print_dir=print_dir)


if __name__ == "__main__":
    parent_dir = get_bms_folder_dir()
    main(parent_dir=parent_dir, print_dir=True)
