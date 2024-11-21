from enum import Enum
import json
import os
from typing import Dict, List, Optional

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
    "435": "gbk",
    "439": "gbk",
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


def is_difficulty_sign(sign: str) -> bool:
    """
    SP ANOTHER
    EZ
    HD
    IN
    AT
    """
    prefix_signs = [
        "SP",
        "DP",
        "PM",
        "5k",
        "7k",
        "9k",
        "14k",
        "5b",
        "7b",
        "9b",
        "14b",
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
    exact_signs = ["B", "N", "H", "A", "I", "SH"]
    for model in prefix_signs:
        if sign.strip().upper().startswith(model.upper()):
            return True
    for model in exact_signs:
        if sign.strip().upper() == model.upper():
            return True
    return False


def deal_with_bms_title(title: str) -> str:
    if title.rstrip().endswith("]"):
        pairs_start_index = title.rfind("[")
        pairs_end_index = pairs_start_index + title[pairs_start_index:].rfind("]")
        if 0 < pairs_start_index < pairs_end_index and is_difficulty_sign(
            title[pairs_start_index + 1 : pairs_end_index]
        ):
            title = title[:pairs_start_index] + title[pairs_end_index + 1 :]
    if title.rstrip().endswith(")"):
        pairs_start_index = title.rfind("(")
        pairs_end_index = pairs_start_index + title[pairs_start_index:].rfind(")")
        if 0 < pairs_start_index < pairs_end_index and is_difficulty_sign(
            title[pairs_start_index + 1 : pairs_end_index]
        ):
            title = title[:pairs_start_index] + title[pairs_end_index + 1 :]
    if title.rstrip().endswith("-"):
        pairs_end_index = title.rfind("-")
        pairs_start_index = title[:pairs_end_index].rfind("-")
        if 0 < pairs_start_index < pairs_end_index and is_difficulty_sign(
            title[pairs_start_index + 1 : pairs_end_index]
        ):
            title = title[:pairs_start_index] + title[pairs_end_index + 1 :]
    return title


class BMSDifficulty(Enum):
    Unknown = 0
    Beginner = 1
    Normal = 2
    Hyper = 3
    Another = 4
    Insane = 5


class BMSInfo:
    def __init__(
        self,
        title: str,
        artist: str,
        genre: str,
        difficulty: BMSDifficulty = BMSDifficulty.Unknown,
        playlevel: int = 0,
    ) -> None:
        self.title = title
        self.artist = artist
        self.genre = genre
        self.difficulty = difficulty
        self.playlevel = playlevel


def parse_bms_file(file_path: str, encoding: Optional[str] = None) -> BMSInfo:
    title = ""
    artist = ""
    genre = ""
    difficulty = BMSDifficulty.Unknown
    playlevel = 0
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
            elif line.startswith("#PLAYLEVEL"):
                playlevel = int(line.replace("#PLAYLEVEL", "").strip())
            elif line.startswith("#DIFFICULTY"):
                difficulty = BMSDifficulty(int(line.replace("#DIFFICULTY", "").strip()))

        title = deal_with_bms_title(title)

    return BMSInfo(title, artist, genre, difficulty, playlevel)


def parse_bmson_file(file_path: str, encoding: Optional[str] = None) -> BMSInfo:
    title = ""
    artist = ""
    genre = ""
    difficulty = BMSDifficulty.Unknown
    playlevel = 0
    with open(file_path, "rb") as file:
        file_bytes = file.read()
        file_str = get_bms_file_str(file_bytes, encoding)

        bmson_info = json.loads(file_str)
        # Get info
        title = bmson_info["info"]["title"]
        artist = bmson_info["info"]["artist"]
        genre = bmson_info["info"]["genre"]
        playlevel = int(bmson_info["info"]["level"])

    return BMSInfo(title, artist, genre, difficulty, playlevel)


def get_dir_bms_info(dir_path: str) -> Optional[BMSInfo]:
    """仅寻找该目录第一层的文件"""
    info: Optional[BMSInfo] = None
    id = os.path.split(dir_path)[-1].split(".")[0]
    encoding = ID_SPECIFIC_ENCODING_TABLE.get(id)
    for file_name in os.listdir(dir_path):
        if info is not None:
            break
        file_path = os.path.join(dir_path, file_name)
        if not os.path.isfile(file_path):
            continue
        if file_name.endswith((".bms", ".bme", ".bml", ".pms")):
            info = parse_bms_file(file_path, encoding)
        elif file_name.endswith((".bmson")):
            info = parse_bmson_file(file_path, encoding)
    return info


def get_dir_bms_info_list(dir_path: str) -> List[BMSInfo]:
    """仅寻找该目录第一层的文件"""
    info_list: List[BMSInfo] = []
    id = os.path.split(dir_path)[-1].split(".")[0]
    encoding = ID_SPECIFIC_ENCODING_TABLE.get(id)
    for file_name in os.listdir(dir_path):
        file_path = os.path.join(dir_path, file_name)
        if not os.path.isfile(file_path):
            continue
        # Parse
        info: Optional[BMSInfo] = None
        if file_name.endswith((".bms", ".bme", ".bml", ".pms")):
            info = parse_bms_file(file_path, encoding)
        elif file_name.endswith((".bmson")):
            info = parse_bmson_file(file_path, encoding)
        # Append
        if info is not None:
            info_list.append(info)
    return info_list
