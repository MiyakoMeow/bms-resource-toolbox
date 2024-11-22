import os
import shutil
from typing import Callable, List, Tuple


RULES: List[Tuple[str, Callable[[str], bool]]] = [
    ("0-9", lambda name: "0" <= name[0].upper() <= "9"),
    ("ABCD", lambda name: "A" <= name[0].upper() <= "D"),
    ("EFGHIJK", lambda name: "E" <= name[0].upper() <= "K"),
    ("LMNOPQ", lambda name: "L" <= name[0].upper() <= "Q"),
    ("RST", lambda name: "R" <= name[0].upper() <= "T"),
    ("UVWXYZ", lambda name: "U" <= name[0].upper() <= "Z"),
    ("假名", lambda name: "ぁ" <= "ん" or "ン" <= name[0].upper() <= "ン"),
    ("+", lambda name: len(name) > 0),
]


def rules_find(name: str) -> str:
    for group_name, func in RULES:
        if not func(name):
            continue
        return group_name
    return "未分类"


def main(root_dir: str):
    root_folder_name = os.path.split(root_dir)[-1]
    if not os.path.isdir(root_dir):
        print(f"{root_dir} is not a dir! Aborting...")
        return
    if root_dir.endswith("]"):
        print(f"{root_dir} endswith ']'. Aborting...")
        return
    parent_dir = os.path.join(root_dir, "..")
    for element_name in os.listdir(root_dir):
        element_path = os.path.join(root_dir, element_name)
        # Find target dir
        rule = rules_find(element_name)
        target_dir = os.path.join(parent_dir, f"{root_folder_name} [{rule}]")
        if not os.path.isdir(target_dir):
            os.mkdir(target_dir)
        # Move
        target_path = os.path.join(target_dir, element_name)
        shutil.move(element_path, target_path)


if __name__ == "__main__":
    root_dir = input("Input the dir to split:")
    main(root_dir)
