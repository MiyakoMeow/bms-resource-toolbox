import os
import shutil
from typing import List


from bms_fs import (
    get_bms_folder_dir,
    get_bms_pack_dir,
    is_dir_having_file,
    move_elements_across_dir,
)
from rawpack import (
    get_num_set_file_names,
    move_out_files_in_folder_in_cache_dir,
    unzip_file_to_cache_dir,
)


def main(pack_dir: str, cache_dir: str, root_dir: str, confirm: bool = False):
    if not os.path.isdir(cache_dir):
        os.mkdir(cache_dir)
    if not os.path.isdir(root_dir):
        os.mkdir(root_dir)

    num_set_file_names: List[str] = get_num_set_file_names(pack_dir)

    if confirm:
        for file_name in num_set_file_names:
            print(f" --> {file_name}")
        selection = input("-> Confirm [y/N]:")
        if selection.lower().startswith("y"):
            return

    for file_name in num_set_file_names:
        file_path = os.path.join(pack_dir, file_name)
        id_str = file_name.split(" ")[0]

        # Prepare an empty cache dir
        cache_dir_path = os.path.join(cache_dir, id_str)

        if os.path.isdir(cache_dir_path) and is_dir_having_file(cache_dir_path):
            shutil.rmtree(cache_dir_path)

        if not os.path.isdir(cache_dir_path):
            os.mkdir(cache_dir_path)

        # Unpack & Copy
        unzip_file_to_cache_dir(file_path, cache_dir_path)

        # Move files in dir
        move_result = move_out_files_in_folder_in_cache_dir(cache_dir_path)
        if not move_result:
            continue

        # Find Existing Target dir
        target_dir_path = None
        for dir_name in os.listdir(root_dir):
            dir_path = os.path.join(root_dir, dir_name)
            if not os.path.isdir(dir_path):
                continue
            if not (
                dir_name.startswith(id_str)
                and (
                    len(dir_name) == len(id_str)
                    or dir_name[len(id_str) :].startswith(".")
                )
            ):
                continue
            target_dir_path = dir_path

        # Create New Target dir
        if target_dir_path is None:
            target_dir_path = os.path.join(root_dir, id_str)

        if not os.path.isdir(target_dir_path):
            os.mkdir(target_dir_path)

        # Move cache to bms dir
        print(f" > Moving files in {cache_dir_path} to {target_dir_path}")
        move_elements_across_dir(cache_dir_path, target_dir_path)
        try:
            os.rmdir(cache_dir_path)
        except FileNotFoundError:
            pass

        # Move File to Another dir
        print(f" > Finish dealing with file: {file_name}")
        used_pack_dir = os.path.join(pack_dir, "BOFTTPacks")
        if not os.path.isdir(used_pack_dir):
            os.mkdir(used_pack_dir)
        shutil.move(file_path, os.path.join(used_pack_dir, file_name))


if __name__ == "__main__":
    pack_dir = get_bms_pack_dir()
    root_dir = get_bms_folder_dir()
    main(
        root_dir=root_dir,
        pack_dir=pack_dir,
        cache_dir=os.path.join(root_dir, "CacheDir"),
    )
