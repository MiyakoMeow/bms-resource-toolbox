import os
import shutil

from fs import get_bms_folder_dir

if __name__ == "__main__":
    root_dir = get_bms_folder_dir()
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
