import os
import shutil

from fs import get_bms_folder_dir, is_dir_having_file


def main(parent_dir: str = ""):
    if len(parent_dir) == 0:
        parent_dir = get_bms_folder_dir()

    for dir_name in os.listdir(parent_dir):
        dir_path = os.path.join(parent_dir, dir_name)
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
