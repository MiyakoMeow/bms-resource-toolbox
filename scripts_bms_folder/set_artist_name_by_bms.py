import os
import shutil
from typing import List, Tuple

from bms import BMSDifficulty, BMSInfo, get_dir_bms_info_list
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
        # Has been set?
        if dir_name.endswith("]"):
            continue
        # Find bmses
        bms_list: List[BMSInfo] = get_dir_bms_info_list(dir_path)
        # Find suitable level 1
        bms_list_lv1 = [
            bms
            for bms in bms_list
            if bms.difficulty != BMSDifficulty.Insane and 1 <= bms.playlevel <= 12
        ]
        if len(bms_list_lv1) > 0:
            bms = bms_list_lv1[0]
            new_dir_name = f"{dir_name} [{bms.artist}]"
            print("- Ready to rename: {} -> {}".format(dir_name, new_dir_name))
            pairs.append((dir_name, new_dir_name))

        elif len(bms_list) > 0:
            bms = bms_list[0]
            new_dir_name = f"{dir_name} [{bms.artist}]"
            print("- Ready to rename: {} -> {}".format(dir_name, new_dir_name))
            pairs.append((dir_name, new_dir_name))
        else:
            print(f"Dir {dir_path} has no bms files!")

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
