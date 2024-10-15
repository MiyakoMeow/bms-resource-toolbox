import json
import os
from typing import Optional

ENCODING = "shift-jis"


class BMSInfo:
    def __init__(self, title: str, artist: str, genre: str) -> None:
        self.title = title
        self.artist = artist
        self.genre = genre


def parse_bms_file(file_path: str) -> BMSInfo:
    title = ""
    artist = ""
    genre = ""
    with open(file_path, "rb") as file:
        file_bytes = file.read()

        file_str = ""
        try:
            file_str = file_bytes.decode(ENCODING, "strict")
        except UnicodeDecodeError:
            try:
                file_str = file_bytes.decode("gb18030", "strict")
            except UnicodeDecodeError:
                try:
                    file_str = file_bytes.decode("utf-8", "strict")
                except UnicodeDecodeError:
                    file_str = file_bytes.decode(ENCODING, "replace")

        for line in file_str.splitlines():
            line = line.strip()
            if line.startswith("#ARTIST"):
                artist = line.replace("#ARTIST", "").strip()
            elif line.startswith("#TITLE"):
                title = line.replace("#TITLE", "").strip()
            elif line.startswith("#GENRE"):
                genre = line.replace("#GENRE", "").strip()
    return BMSInfo(title, artist, genre)


def parse_bmson_file(file_path: str) -> BMSInfo:
    title = ""
    artist = ""
    genre = ""
    with open(file_path, "rb") as file:
        file_bytes = file.read()

        file_str = ""
        try:
            file_str = file_bytes.decode(ENCODING, "strict")
        except UnicodeDecodeError:
            try:
                file_str = file_bytes.decode("gb18030", "strict")
            except UnicodeDecodeError:
                try:
                    file_str = file_bytes.decode("utf-8", "strict")
                except UnicodeDecodeError:
                    file_str = file_bytes.decode(ENCODING, "replace")

        bmson_info = json.loads(file_str)
        # Get info
        title = bmson_info["info"]["title"]
        artist = bmson_info["info"]["artist"]
        genre = bmson_info["info"]["genre"]

    return BMSInfo(title, artist, genre)


def get_dir_bms_info(dir_path: str) -> Optional[BMSInfo]:
    info: Optional[BMSInfo] = None
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
    return info
