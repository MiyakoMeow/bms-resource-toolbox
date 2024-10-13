import os
import os.path
import shutil
import zipfile

import py7zr
import rarfile

from bms_fs import move_files_across_dir

BOFTT_PACK_DIR = os.environ.get("BOFTT_PACK_DIR")
if BOFTT_PACK_DIR is None:
    BOFTT_PACK_DIR = os.path.abspath(".")

BOFTT_DIR = os.environ.get("BOFTT_DIR")
if BOFTT_DIR is None:
    BOFTT_DIR = os.path.abspath(".")

if __name__ == "__main__":
    print("Set default pack dir by env BOFTT_PACK_DIR")
    pack_dir = input(f"Input root dir of bms pack dirs (Default: {BOFTT_PACK_DIR}):")
    if len(pack_dir.strip()) == 0:
        pack_dir = BOFTT_PACK_DIR

    print("Set default dir by env BOFTT_DIR")
    root_dir = input(f"Input root dir of bms dirs (Default: {BOFTT_DIR}):")
    if len(root_dir.strip()) == 0:
        root_dir = BOFTT_DIR

    for file_name in os.listdir(pack_dir):
        file_path = f"{pack_dir}/{file_name}"
        if not os.path.isfile(file_path):
            continue
        id_str = file_name.split(" ")[0]
        if not id_str.isdigit():
            continue

        # Create a cache dir
        cache_dir_path = f"{pack_dir}/{id_str}"
        os.mkdir(cache_dir_path)

        # Unpack & Copy
        if file_name.endswith(".zip"):
            print(f"Extracting {file_path} to {cache_dir_path} (zip)")
            zip_file = zipfile.ZipFile(file_path)
            zip_file.extractall(cache_dir_path)
            zip_file.close()
        elif file_name.endswith(".7z"):
            print(f"Extracting {file_path} to {cache_dir_path} (7z)")
            zip_file = py7zr.SevenZipFile(file_path)
            zip_file.extractall(cache_dir_path)
            zip_file.close()
        elif file_name.endswith(".rar"):
            print(f"Extracting {file_path} to {cache_dir_path} (RAR)")
            zip_file = rarfile.RarFile(file_path)
            zip_file.extractall(cache_dir_path)
            zip_file.close()
        else:
            target_file_path = f"{cache_dir_path}/{"".join(file_name.split(" ")[1:])}"
            print(f"Coping {file_path} to {target_file_path}")
            shutil.copy(file_path, target_file_path)

        # Move cache to bms dir
        target_dir_path = None
        for dir_name in os.listdir(root_dir):
            dir_path = f"{root_dir}/{dir_name}"
            if not os.path.isdir(dir_path):
                continue
            if not dir_name.startswith(id_str):
                continue
            target_dir_path = dir_path

        if target_dir_path is None:
            target_dir_path = f"{root_dir}/{id_str}"

        if not os.path.isdir(target_dir_path):
            os.mkdir(target_dir_path)

        print(f"Moving files in {cache_dir_path} to {target_dir_path}")
        move_files_across_dir(cache_dir_path, target_dir_path)
        os.rmdir(cache_dir_path)

        # Move File to Another dir
        print(f"Finish dealing with file: {file_name}")
        shutil.move(file_path, f"{pack_dir}/BOFTTPacks/{file_name}")
