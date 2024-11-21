import os
from typing import List, Tuple

from bms_fs import get_bms_folder_dir, move_elements_across_dir


def merge_folders(root_dir: str):
    dir_names: List[str] = [
        dir_name
        for dir_name in os.listdir(root_dir)
        if os.path.isdir(os.path.join(root_dir, dir_name))
    ]

    pairs: List[Tuple[str, str]] = []

    for dir_name in dir_names:
        dir_path = os.path.join(root_dir, dir_name)
        if not os.path.isdir(dir_path):
            continue
        # Situation 1: endswith "]"
        if dir_name.endswith("]"):
            # Find dir_name_without_artist
            dir_name_mps_i = dir_name.rfind("[")
            if dir_name_mps_i == -1:
                continue
            dir_name_without_artist = dir_name[: dir_name_mps_i - 1]
            if len(dir_name_without_artist) == 0:
                continue
            # Check folder
            dir_path_without_artist = os.path.join(root_dir, dir_name_without_artist)
            if not os.path.isdir(dir_path_without_artist):
                continue
            # Check has another folders
            dir_names_with_starter = [
                dir_name
                for dir_name in dir_names
                if dir_name.startswith(f"{dir_name_without_artist} [")
            ]
            if len(dir_names_with_starter) > 2:
                print(
                    " !_! {} have more then 2 folders! {}".format(
                        dir_name_without_artist, dir_names_with_starter
                    )
                )
                continue

            # Print
            print("- Find Dir pair: {} <- {}".format(dir_name, dir_name_without_artist))
            pairs.append((dir_name, dir_name_without_artist))

    selection = input("Do transfering? [y/N]:")
    if not selection.lower().startswith("y"):
        print("Aborted.")
        return

    for target_dir_name, from_dir_name in pairs:
        from_dir_path = os.path.join(root_dir, from_dir_name)
        target_dir_path = os.path.join(root_dir, target_dir_name)
        move_elements_across_dir(from_dir_path, target_dir_path)
        os.rmdir(from_dir_path)


def main(
    root_dir: str = "",
):
    if len(root_dir) == 0:
        root_dir = get_bms_folder_dir()
    merge_folders(root_dir)


if __name__ == "__main__":
    main()
