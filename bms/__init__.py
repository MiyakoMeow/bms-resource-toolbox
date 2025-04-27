import os
from typing import List, Optional

from bms.encoding import BOFTT_ID_SPECIFIC_ENCODING_TABLE
from bms.parse import BMSInfo, parse_bms_file, parse_bmson_file
from bms.work import extract_work_name


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
