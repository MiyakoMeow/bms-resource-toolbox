import os
import shutil
from typing import List


def rename_file(dir: str, file_name: str, input_num: int):
    file_path = os.path.join(dir, file_name)
    new_file_name = f"{input_num} {file_name}"
    new_file_path = os.path.join(dir, new_file_name)
    shutil.move(file_path, new_file_path)
    print(f"Rename {file_name} to {new_file_name}.")
    print()


def cycle(
    dir: str,
    allow_ext: List[str] = [],
    disallow_ext: List[str] = [],
    allow_others: bool = True,
):
    file_names = []
    for file_name in os.listdir(dir):
        file_path = os.path.join(dir, file_name)
        # Not File?
        if not os.path.isfile(file_path):
            continue
        # Has been numbered?
        if file_name.split()[0].isdigit():
            continue
        # Linux: Has Partial File?
        part_file_path = f"{file_path}.part"
        if os.path.isfile(part_file_path):
            continue
        # Linux: Empty File?
        if os.path.getsize(file_path) == 0:
            continue
        # Is Allowed?
        file_ext = file_name.rsplit(".", 1)[-1]
        allowed = allow_others
        if file_ext in allow_ext:
            allowed = True
        elif file_ext in disallow_ext:
            allowed = False
        if not allowed:
            continue
        file_names.append(file_name)

    # Print Selections
    print(f"Here are files in {dir}:")
    for i, file_name in enumerate(file_names):
        print(f" - {i}: {file_name}")

    print("Input a number: to set num [0] to the first selection.")
    print("Input two numbers: to set num [1] to the selection in index [0].")
    input_str = input("Input:")
    input_str_split = input_str.split()
    if len(input_str_split) == 2:
        file_name = file_names[int(input_str_split[0])]
        input_num = int(input_str_split[1])
        rename_file(dir, file_name, input_num)
    elif len(input_str_split) == 1:
        file_name = file_names[0]
        input_num = int(input_str_split[0])
        rename_file(dir, file_name, input_num)
    else:
        print("Invaild input.")
        print()


def main():
    dir = input(f"Input dir (Default: {os.path.abspath(".")}):")
    if len(dir.strip()) == 0:
        dir = "."
    while True:
        cycle(
            dir,
            allow_ext=["zip", "7z", "rar", "mp4", "bms", "bme", "bml", "pms"],
            disallow_ext=[],
            allow_others=False,
        )


if __name__ == "__main__":
    main()
