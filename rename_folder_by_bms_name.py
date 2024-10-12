import json
import os
import os.path
import shutil


BOFTT_DIR = os.environ.get("BOFTT_DIR")
if BOFTT_DIR is None:
    BOFTT_DIR = os.path.abspath(".")

ENCODING = "shift-jis"


class BMSInfo:
    def __init__(self, title: str, artist: str) -> None:
        self.title = title
        self.artist = artist


def parse_bms_file(file_path: str) -> BMSInfo:
    title = ""
    artist = ""
    with open(file_path, "rb") as file:
        file_bytes = file.read()
        file_str = file_bytes.decode(ENCODING)

        for line in file_str.splitlines():
            line = line.strip()
            if line.startswith("#ARTIST"):
                artist = line.replace("#ARTIST", "").strip()
            elif line.startswith("#TITLE"):
                title = line.replace("#TITLE", "").strip()
    return BMSInfo(title, artist)


def parse_bmson_file(file_path: str) -> BMSInfo:
    title = ""
    artist = ""
    with open(file_path, "rb") as file:
        file_bytes = file.read()
        file_str = file_bytes.decode(ENCODING)
        bmson_info = json.loads(file_str)
        # Get info
        title = bmson_info["info"]["title"]
        artist = bmson_info["info"]["artist"]

    return BMSInfo(title, artist)


def rename_dir(dir_path: str):
    if not dir_path.split("/")[-1].split("\\")[-1].strip().isdigit():
        # print(f"{dir_path} has been renamed! Skipping...")
        return
    info = None
    for root, _, files in os.walk(dir_path):
        if info is not None:
            break
        for file_name in files:
            if info is not None:
                break
            file_path = f"{root}/{file_name}"
            if file_name.endswith((".bms", ".bme", ".bml", ".pms")):
                info = parse_bms_file(file_path)
            elif file_name.endswith((".bmson")):
                info = parse_bmson_file(file_path)
    if info is None:
        # print(f"{dir_path} has no bms/bmson files!")
        return
    # Rename
    print(f"{dir_path} found bms title: {info.title} artist: {info.artist}")
    new_dir_path = f"{dir_path}. {info.title} [{info.artist}]"
    shutil.move(dir_path, new_dir_path)
    # Move out files
    dir_path = new_dir_path
    dir_inner_list = os.listdir(dir_path)
    if len(dir_inner_list) > 1:
        return
    dir_inner_path = f"{dir_path}/{dir_inner_list[0]}"
    print(f"Moving files in {dir_inner_path} to parent folder")
    for file_name in os.listdir(dir_inner_path):
        ori_path = f"{dir_inner_path}/{file_name}"
        dst_path = f"{dir_path}/{file_name}"
        shutil.move(ori_path, dst_path)
    os.rmdir(dir_inner_path)


if __name__ == "__main__":
    print("Set default dir by env BOFTT_DIR")
    root_dir = input(f"Input root dir of bms dirs (Default: {BOFTT_DIR}):")
    if len(root_dir.strip()) == 0:
        root_dir = BOFTT_DIR
    for id, dir_name in enumerate(os.listdir(root_dir)):
        if not os.path.isdir(dir_name):
            continue
        dir_path = f"{root_dir}/{dir_name}"
        rename_dir(dir_path)
