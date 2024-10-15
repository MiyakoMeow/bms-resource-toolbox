import os
import shutil

BOFTT_DIR = os.environ.get("BOFTT_DIR")
if BOFTT_DIR is None:
    BOFTT_DIR = os.path.abspath(".")

if __name__ == "__main__":
    print("Set default dir by env BOFTT_DIR")
    root_dir = input(f"Input root dir of bms dirs (Default: {BOFTT_DIR}):")
    if len(root_dir.strip()) == 0:
        root_dir = BOFTT_DIR
    for dir_name in os.listdir(root_dir):
        dir_path = f"{root_dir}/{dir_name}"
        if not os.path.isdir(dir_path):
            continue
        new_dir_name = dir_name.split(" ")[0]
        new_dir_path = f"{root_dir}/{new_dir_name}"
        if dir_name == new_dir_name:
            continue
        print(f"Rename {dir_name} to {new_dir_name}")
        shutil.move(dir_path, new_dir_path)
