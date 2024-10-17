import json
import os
from typing import Dict, Optional

ENCODINGS = [
    "shift-jis",
    "gb2312",
    "utf-8",
    "shift-jis-2004",
    "gb18030",
    "shift-jisx0213",
]

ID_SPECIFIC_ENCODING_TABLE: Dict[str, str] = {
    "134": "utf-8",
    "191": "gbk",
    # 159 bms文件本身有编码问题
}


def get_bms_file_str(file_bytes: bytes, encoding: Optional[str] = None) -> str:
    file_str = ""
    if encoding is not None:
        file_str = file_bytes.decode(encoding, "replace")
        return file_str
    done = False
    for encoding in ENCODINGS:
        try:
            file_str = file_bytes.decode(encoding, "strict")
            done = True
        except UnicodeDecodeError:
            pass
        if done:
            break
    if not done:
        file_str = file_bytes.decode("utf-8", "replace")

    return file_str


class BMSInfo:
    def __init__(self, title: str, artist: str, genre: str) -> None:
        self.title = title
        self.artist = artist
        self.genre = genre


def parse_bms_file(file_path: str, encoding: Optional[str] = None) -> BMSInfo:
    title = ""
    artist = ""
    genre = ""
    with open(file_path, "rb") as file:
        file_bytes = file.read()
        file_str = get_bms_file_str(file_bytes, encoding)

        for line in file_str.splitlines():
            line = line.strip()
            if line.startswith("#ARTIST"):
                artist = line.replace("#ARTIST", "").strip()
            elif line.startswith("#TITLE"):
                title = line.replace("#TITLE", "").strip()
            elif line.startswith("#GENRE"):
                genre = line.replace("#GENRE", "").strip()
    return BMSInfo(title, artist, genre)


def parse_bmson_file(file_path: str, encoding: Optional[str] = None) -> BMSInfo:
    title = ""
    artist = ""
    genre = ""
    with open(file_path, "rb") as file:
        file_bytes = file.read()
        file_str = get_bms_file_str(file_bytes, encoding)

        bmson_info = json.loads(file_str)
        # Get info
        title = bmson_info["info"]["title"]
        artist = bmson_info["info"]["artist"]
        genre = bmson_info["info"]["genre"]

    return BMSInfo(title, artist, genre)


def get_dir_bms_info(dir_path: str) -> Optional[BMSInfo]:
    """仅寻找该目录第一层的文件"""
    info: Optional[BMSInfo] = None
    id = dir_path.split("/")[-1].split(".")[0]
    encoding = ID_SPECIFIC_ENCODING_TABLE.get(id)
    for file_name in os.listdir(dir_path):
        if info is not None:
            break
        file_path = f"{dir_path}/{file_name}"
        if not os.path.isfile(file_path):
            continue
        if file_name.endswith((".bms", ".bme", ".bml", ".pms")):
            info = parse_bms_file(file_path, encoding)
        elif file_name.endswith((".bmson")):
            info = parse_bmson_file(file_path, encoding)
    return info
