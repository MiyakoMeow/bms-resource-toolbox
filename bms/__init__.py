from enum import Enum
from collections import defaultdict
import json
import os
from typing import Any, Dict, List, Optional, Tuple
from dataclasses import dataclass, field

from bms.encodings import PriorityDecoder

ENCODINGS = [
    "shift-jis",
    "shift-jis-2004",
    "gb2312",
    "utf-8",
    "gb18030",
    "shift-jisx0213",
]

BOFTT_ID_SPECIFIC_ENCODING_TABLE: Dict[str, str] = {
    "134": "utf-8",
    "191": "gbk",
    "435": "gbk",
    "439": "gbk",
    # 159 bms文件本身有编码问题
}


def get_bms_file_str(file_bytes: bytes, encoding: Optional[str] = None) -> str:
    file_str = ""
    encoding_priority = ENCODINGS
    if encoding:
        encoding_priority.insert(0, encoding)
    decoder = PriorityDecoder(encoding_priority)
    try:
        file_str = decoder.decode(file_bytes, errors="strict")
    except UnicodeDecodeError:
        file_str = file_bytes.decode("utf-8", errors="ignore")

    return file_str


class BMSDifficulty(Enum):
    Unknown = 0
    Beginner = 1
    Normal = 2
    Hyper = 3
    Another = 4
    Insane = 5


@dataclass
class BMSInfo:
    title: str
    artist: str
    genre: str
    difficulty: BMSDifficulty = BMSDifficulty.Unknown
    playlevel: int = 0
    bmp_formats: List[str] = field(default_factory=list)


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
                    now = now.get(i)
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


def get_dir_bms_list(dir_path: str) -> List[BMSInfo]:
    """仅寻找该目录第一层的文件"""
    info_list: List[BMSInfo] = []
    # For BOFTT
    id = os.path.split(dir_path)[-1].split(".")[0]
    encoding = BOFTT_ID_SPECIFIC_ENCODING_TABLE.get(id)
    # Scan
    for file_name in os.listdir(dir_path):
        file_path = os.path.join(dir_path, file_name)
        # Parse
        info: Optional[BMSInfo] = None
        if file_name.lower().endswith(
            (".bms", ".bme", ".bml", ".pms")
        ) and os.path.isfile(file_path):
            info = parse_bms_file(file_path, encoding)
        elif file_name.lower().endswith((".bmson")) and os.path.isfile(file_path):
            info = parse_bmson_file(file_path, encoding)
        # Append
        if info is not None:
            info_list.append(info)
    return info_list


def get_dir_bms_info(bms_dir_path: str) -> Optional[BMSInfo]:
    # Find bmses
    bms_list: List[BMSInfo] = get_dir_bms_list(bms_dir_path)
    if len(bms_list) == 0:
        return None
    # Split title
    title = extract_work_name([bms.title for bms in bms_list])
    if title.endswith("-") and title.count("-") % 2 != 0 and title[-2].isspace():
        title = title[:-1].strip()
    artist = extract_work_name(
        [bms.artist for bms in bms_list],
        remove_tailing_sign_list=[
            "/",
            ":",
            "：",
            "-",
            "obj",
            "obj.",
            "Obj",
            "Obj.",
            "OBJ",
            "OBJ.",
        ],
    )
    genre = extract_work_name([bms.genre for bms in bms_list])
    return BMSInfo(title, artist, genre)


def extract_work_name(
    titles: List[str],
    remove_unclosed_pair: bool = True,
    remove_tailing_sign_list: List[str] = [],
) -> str:
    """
    从多个BMS文件标题中提取共同的作品名（改进版）

    :param titles: 包含多个BMS标题的列表
    :return: 提取出的共同作品名（经过后处理）
    """
    # 统计所有可能前缀的出现次数
    prefix_counts = defaultdict(int)
    for title in titles:
        for i in range(1, len(title) + 1):
            prefix = title[:i]
            prefix_counts[prefix] += 1

    if not prefix_counts:
        return ""

    max_count = max(prefix_counts.values())

    candidates = [
        (prefix, count)
        for prefix, count in prefix_counts.items()
        if count >= max_count * 0.5  # 超过一半就可以算上
    ]

    # 排序规则：优先长度降序，其次次数降序，最后字典序升序
    candidates.sort(key=lambda x: (-len(x[0]), -x[1], x[0]))

    # 提取最优候选
    best_candidate = candidates[0][0] if candidates else ""

    # 后处理：移除未闭合括号及其后续内容
    return _extract_work_name_post_process(
        best_candidate,
        remove_unclosed_pair=remove_unclosed_pair,
        remove_tailing_sign_list=remove_tailing_sign_list,
    )


def _extract_work_name_post_process(
    s: str, remove_unclosed_pair: bool = True, remove_tailing_sign_list: List[str] = []
) -> str:
    """
    后处理函数：移除未闭合括号及其后续内容

    :param s: 原始字符串
    :return: 处理后的字符串
    """

    # 清除前后空格
    s = s.strip()

    while True:
        triggered = False
        if remove_unclosed_pair:
            stack: List[Tuple[str, int]] = []

            # 遍历字符串记录括号状态
            pairs = [
                ("(", ")"),
                ("[", "]"),
                ("{", "}"),
                ("（", "）"),
                ("［", "］"),
                ("｛", "｝"),
                ("【", "】"),
            ]
            for i, c in enumerate(s):
                for p_open, p_close in pairs:
                    if c == p_open:
                        stack.append((c, i))  # 记录括号类型和位置
                    if c == p_close and stack and stack[-1][0] == p_open:
                        stack.pop()

            # 如果存在未闭合括号
            if stack:
                last_unmatched_pos = stack[-1][1]
                s = s[:last_unmatched_pos].rstrip()  # 截断并移除末尾空格
                triggered = True

        for sign in remove_tailing_sign_list:
            if s.endswith(sign):
                s = s[: -len(sign)].rstrip()
                triggered = True
        # 没触发？
        if not triggered:
            break

    return s
