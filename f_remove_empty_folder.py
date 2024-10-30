import os
import shutil

from bms_fs import get_bms_folder_dir


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


def main(bms_dir: str = ""):
    if len(bms_dir) == 0:
        bms_dir = get_bms_folder_dir()

    for dir_name in os.listdir(bms_dir):
        dir_path = os.path.join(bms_dir, dir_name)
        if not os.path.isdir(dir_path):
            continue
        if not is_dir_having_file(dir_path):
            print(f"Remove empty dir: {dir_path}")
            shutil.rmtree(dir_path)


if __name__ == "__main__":
    main()
