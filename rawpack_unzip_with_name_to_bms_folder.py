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
    move_out_files_in_folder_in_cache_dir,
    unzip_file_to_cache_dir,
)


def main(pack_dir: str, cache_dir: str, root_dir: str, confirm: bool = False):
    if not os.path.isdir(cache_dir):
        os.mkdir(cache_dir)
    if not os.path.isdir(root_dir):
        os.mkdir(root_dir)

    num_set_file_names: List[str] = [
        file_name
        for file_name in os.listdir(pack_dir)
        if os.path.isfile(os.path.join(pack_dir, file_name))
        and (
            file_name.endswith(".zip")
            or file_name.endswith(".7z")
            or file_name.endswith(".rar")
        )
    ]

    if confirm:
        for file_name in num_set_file_names:
            print(f" --> {file_name}")
        selection = input("-> Confirm [y/N]:")
        if selection.lower().startswith("y"):
            return

    for file_name in num_set_file_names:
        file_path = os.path.join(pack_dir, file_name)
        file_name_without_ext = file_name[: -len(file_name.rsplit(".", 1)[-1]) - 1]

        # Prepare an empty cache dir
        cache_dir_path = os.path.join(cache_dir, file_name_without_ext)

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

        target_dir_path = os.path.join(root_dir, file_name_without_ext)

        # Create New Target dir
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
    root_dir = get_bms_folder_dir()
    pack_dir = get_bms_pack_dir()
    main(
        root_dir=root_dir,
        pack_dir=pack_dir,
        cache_dir=os.path.join(root_dir, "CacheDir"),
    )
