import os
import os.path
import shutil
from typing import Optional

from bms import get_dir_bms_info, BMSInfo
from bms_fs import get_bms_folder_dir, get_vaild_fs_name


def is_difficulty_sign(sign: str) -> bool:
    """
    SP ANOTHER
    EZ
    HD
    IN
    AT
    """
    sign = sign.strip().upper()
    sign_models = [
        "SP",
        "DP",
        "7k",
        "14k",
        "9k",
        "beginner",
        "normal",
        "hyper",
        "another",
        "light",
        "main",
        "hard",
        "EZ",
        "HD",
        "IN",
        "AT",
    ]
    for model in sign_models:
        if sign.startswith(model.upper()):
            return True
    return False


def deal_with_dir(dir_path: str):
    if not dir_path.split("/")[-1].split("\\")[-1].strip().isdigit():
        # print(f"{dir_path} has been renamed! Skipping...")
        return
    info: Optional[BMSInfo] = get_dir_bms_info(dir_path)
    if info is None:
        print(f"{dir_path} has no bms/bmson files!")
        return

    # # Scan folder
    # file_count = 0
    # folder_count = 0
    # only_folder_name = None
    # for inner_element_name in os.listdir(dir_path):
    #     inner_path = f"{dir_path}/{inner_element_name}"
    #     if os.path.isfile(inner_path):
    #         file_count += 1
    #     elif os.path.isdir(inner_path):
    #         folder_count += 1
    #         only_folder_name = inner_element_name
    #
    # # Check folder
    # if folder_count == 0 and file_count == 0:
    #     print(f"{dir_path} is empty!")
    #     return
    # if (folder_count == 0 or folder_count == 1) and 1 < file_count <= 10:
    #     print(
    #         f"{dir_path} has no enough files, is likely not a bms folder, or not arranged!"
    #     )
    #     return
    # if folder_count > 1:
    #     print(f"{dir_path} has extra folders!")
    #     return
    #
    # # Move out files
    # if only_folder_name is not None:
    #     dir_inner_path = f"{dir_path}/{only_folder_name}"
    #     print(f"Moving files in {dir_inner_path} to parent folder")
    #     move_files_across_dir(dir_inner_path, dir_path)
    #     os.rmdir(dir_inner_path)

    # Deal with info
    print(f"{dir_path} found bms title: {info.title} artist: {info.artist}")
    title = info.title
    artist = info.artist
    if title.rstrip().endswith("]"):
        pairs_start_index = title.rfind("[")
        pairs_end_index = title.rfind("]")
        if is_difficulty_sign(title[pairs_start_index + 1 : pairs_end_index]):
            title = title[:pairs_start_index] + title[pairs_end_index + 1 :]
    if title.rstrip().endswith(")"):
        pairs_start_index = title.rfind("(")
        pairs_end_index = title.rfind(")")
        if is_difficulty_sign(title[pairs_start_index + 1 : pairs_end_index]):
            title = title[:pairs_start_index] + title[pairs_end_index + 1 :]

    # Rename
    new_dir_path = (
        f"{dir_path}. {get_vaild_fs_name(title)} [{get_vaild_fs_name(artist)}]"
    )
    shutil.move(dir_path, new_dir_path)


if __name__ == "__main__":
    root_dir = get_bms_folder_dir()
    for dir_name in os.listdir(root_dir):
        dir_path = f"{root_dir}/{dir_name}"
        if not os.path.isdir(dir_path):
            continue
        deal_with_dir(dir_path)
