from enum import Enum
import json
import os
from typing import Any, Dict, List, Optional

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
        bmp_formats: List[str] = [],
    ) -> None:
        self.title = title
        self.artist = artist
        self.genre = genre
        self.difficulty = difficulty
        self.playlevel = playlevel
        self.bmp_formats = bmp_formats


def parse_bms_file(file_path: str, encoding: Optional[str] = None) -> BMSInfo:
    title = ""
    artist = ""
    genre = ""
    difficulty = BMSDifficulty.Unknown
    playlevel = 0
    ext_list = []
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
                value_str = line.replace("#PLAYLEVEL", "").strip()
                if len(value_str) > 0 and value_str.isdecimal():
                    playlevel_float = float(value_str)
                    playlevel = (
                        int(playlevel_float) if 0.0 <= playlevel_float <= 99.0 else -1
                    )
            elif line.startswith("#DIFFICULTY"):
                value_str = line.replace("#DIFFICULTY", "").strip()
                if len(value_str) > 0 and value_str.isdecimal():
                    value = int(float(value_str))
                    difficulty = (
                        BMSDifficulty(value)
                        if 0 <= value <= 5
                        else BMSDifficulty.Unknown
                    )
            elif line.startswith("#BMP"):
                value_str = line.replace("#BMP", "").strip()
                ext = os.path.splitext(value_str)[1]
                if ext is not None:
                    ext_list.append(ext)

        title = deal_with_bms_title(title)

    return BMSInfo(title, artist, genre, difficulty, playlevel, ext_list)


def parse_bmson_file(file_path: str, encoding: Optional[str] = None) -> BMSInfo:
    title = ""
    artist = ""
    genre = ""
    difficulty = BMSDifficulty.Unknown
    playlevel = 0
    with open(file_path, "rb") as file:
        file_bytes = file.read()
        file_str = get_bms_file_str(file_bytes, encoding)

        try:
            bmson_info: Dict[Any, Any] = json.loads(file_str)
        except json.JSONDecodeError:
            print(f" !_!: Json Decode Error! {file_path}")
            return BMSInfo("Error", "Error", "Error")

        # Get info
        def dict_get(dict: Dict[Any, Any], *info) -> Optional[Any]:
            now = dict
            for i in info:
                if now is not None:
                    now = now[i]
            return now

        title = dict_get(bmson_info, "info", "title") or ""
        artist = dict_get(bmson_info, "info", "artist") or ""
        genre = dict_get(bmson_info, "info", "genre") or ""
        playlevel = int(dict_get(bmson_info, "info", "level") or 0)
        ext_list = []
        bga_headers = dict_get(bmson_info, "bga", "bga_header")
        if bga_headers is not None:
            for bga_header in bga_headers:
                file_name = bga_header["name"]
                ext = os.path.splitext(file_name)[1]
                if ext is not None:
                    ext_list.append(ext)

    return BMSInfo(title, artist, genre, difficulty, playlevel, ext_list)


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
