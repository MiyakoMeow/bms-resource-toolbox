import os
import shutil

from bms import BMSInfo


def get_vaild_fs_name(ori_name: str) -> str:
    return (
        ori_name.replace(":", "：")
        .replace("/", "←")
        .replace("\\", "→")
        .replace(" & ", " and ")
        .replace("&", " and ")
        .replace(" | ", " or ")
        .replace("|", " or ")
        .replace("?", "？")
        .replace("!", "！")
        .replace(",", "，")
        .replace("<", "《")
        .replace(">", "》")
        .replace("*", "☆")
        .replace('"', "“")
    )


def get_folder_name(id: str, info: BMSInfo) -> str:
    return f"{id}. {get_vaild_fs_name(info.title)} [{get_vaild_fs_name(info.artist)}]"


def move_files_across_dir(dir_path_ori: str, dir_path_dst: str):
    for file_name in os.listdir(dir_path_ori):
        ori_path = f"{dir_path_ori}/{file_name}"
        dst_path = f"{dir_path_dst}/{file_name}"
        shutil.move(ori_path, dst_path)
