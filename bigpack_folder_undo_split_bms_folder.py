import os
from typing import List, Tuple

from bms_fs import move_elements_across_dir


def main(root_dir: str):
    if not os.path.isdir(root_dir):
        os.mkdir(root_dir)
    root_folder_name = os.path.split(root_dir)[-1]
    parent_dir = os.path.join(root_dir, "..")
    pairs: List[Tuple[str, str]] = []
    for folder_name in os.listdir(parent_dir):
        folder_path = os.path.join(parent_dir, folder_name)
        if folder_name.startswith(f"{root_folder_name} [") and folder_name.endswith(
            "]"
        ):
            print(f" - {root_dir} <- {folder_path}")
            pairs.append((folder_path, root_dir))

    confirm = input("Confirm? [y/N]")
    if not confirm.lower().startswith("y"):
        return

    for from_dir, to_dir in pairs:
        move_elements_across_dir(from_dir, to_dir)


if __name__ == "__main__":
    root_dir = input("Input the target dir to merge:")
    main(root_dir)
