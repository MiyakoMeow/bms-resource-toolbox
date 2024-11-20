import os
import shutil
from typing import List, Tuple

from bms_fs import get_bms_folder_dir


def set_folder_artist_name(root_dir: str):
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
        if not dir_name.endswith("]"):
            # TODO
            pass

    selection = input("Do transfering? [y/N]:")
    if not selection.lower().startswith("y"):
        print("Aborted.")
        return

    for from_dir_name, target_dir_name in pairs:
        from_dir_path = os.path.join(root_dir, from_dir_name)
        target_dir_path = os.path.join(root_dir, target_dir_name)
        shutil.move(from_dir_path, target_dir_path)


def main(
    root_dir: str = "",
):
    if len(root_dir) == 0:
        root_dir = get_bms_folder_dir()
    set_folder_artist_name(root_dir)


if __name__ == "__main__":
    main()
