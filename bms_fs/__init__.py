from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor, as_completed
import os
import shutil
from typing import List, Optional, Tuple

from bms import BMSInfo


"""
DIR
"""

_BMS_FOLDER: Optional[str] = None
_BMS_PACK_DIR: Optional[str] = None


def get_bms_folder_dir(tips: bool = True, use_default: bool = True) -> str:
    global _BMS_FOLDER
    if _BMS_FOLDER is not None:
        return _BMS_FOLDER
    BMS_FOLDER = os.environ.get("BMS_FOLDER")
    if BMS_FOLDER is None:
        BMS_FOLDER = os.path.abspath(".")
    if tips:
        print("Set default dir by env BMS_FOLDER")
        print(f"Input root dir path of bms dirs (Default: {BMS_FOLDER}):", end="")
    root_dir = input()
    if len(root_dir.strip()) == 0:
        if use_default:
            root_dir = BMS_FOLDER
        else:
            raise Exception("Default Value Disabled.")
    _BMS_FOLDER = root_dir
    return _BMS_FOLDER


def get_bms_pack_dir(tips: bool = True, use_default: bool = True) -> str:
    global _BMS_PACK_DIR
    if _BMS_PACK_DIR is not None:
        return _BMS_PACK_DIR
    BMS_PACK_DIR = os.environ.get("BMS_PACK_DIR")
    if BMS_PACK_DIR is None:
        BMS_PACK_DIR = os.path.abspath(".")
    if tips:
        print("Set default pack dir by env BMS_PACK_DIR")
        print(f"Input dir path of bms packs (Default: {BMS_PACK_DIR}):", end="")
    root_dir = input()
    if len(root_dir.strip()) == 0:
        if use_default:
            root_dir = BMS_PACK_DIR
        else:
            raise Exception("Default Value Disabled.")
    _BMS_PACK_DIR = root_dir
    return _BMS_PACK_DIR


"""
FS
"""


def get_vaild_fs_name(ori_name: str) -> str:
    """
    Signs:
    ：＼／＊？＂＜＞｜
    """
    return (
        ori_name.replace(":", "：")
        .replace("\\", "＼")
        .replace("/", "／")
        .replace("*", "＊")
        .replace("?", "？")
        .replace("!", "！")
        .replace('"', "＂")
        .replace("<", "＜")
        .replace(">", "＞")
        .replace("|", "｜")
    )


def get_folder_name(id: str, info: BMSInfo) -> str:
    return f"{id}. {get_vaild_fs_name(info.title)} [{get_vaild_fs_name(info.artist)}]"


def move_elements_across_dir(
    dir_path_ori: str,
    dir_path_dst: str,
    print_info: bool = False,
    replace: bool = True,
    # TODO: More smart
    replace_new_file_if_unique: bool = False,
):
    if not os.path.isdir(dir_path_ori):
        return
    if not os.path.isdir(dir_path_dst):
        os.mkdir(dir_path_dst)

    next_folder_paths: List[Tuple[str, str]] = []

    def is_same_content(file_a: str, file_b: str) -> bool:
        with open(file_a, "rb") as fa:
            with open(file_b, "rb") as fb:
                ca = fa.read()
                cb = fb.read()
                return ca == cb

    def move_action(ori_path: str, dst_path: str):
        if print_info:
            print(f" - Moving from {ori_path} to {dst_path}")
        # Move
        if os.path.isfile(ori_path):
            # Replace?
            if replace:
                if not replace_new_file_if_unique:
                    shutil.move(ori_path, dst_path)
                elif is_same_content(ori_path, dst_path):
                    shutil.move(ori_path, dst_path)
            # Not exists? Move
            elif not os.path.isfile(dst_path):
                shutil.move(ori_path, dst_path)
        elif os.path.isdir(ori_path):
            # Directly move dir
            if not os.path.isdir(dst_path):
                shutil.move(ori_path, dst_path)
            else:
                # Add child dir
                next_folder_paths.append((ori_path, dst_path))

    with ThreadPoolExecutor(max_workers=4) as executor:
        # 提交任务
        dir_lists: List[Tuple[str, str]] = [
            (
                os.path.join(dir_path_ori, element_name),
                os.path.join(dir_path_dst, element_name),
            )
            for element_name in os.listdir(dir_path_ori)
        ]
        futures = [
            executor.submit(
                move_action,
                path_ori,
                path_dst,
            )
            for path_ori, path_dst in dir_lists
        ]
        # 等待任务完成
        for _ in as_completed(futures):
            pass

    # Next Level
    for ori_path, dst_path in next_folder_paths:
        move_elements_across_dir(ori_path, dst_path, print_info, replace)

    # Clean Source
    if replace or not is_dir_having_file(dir_path_ori):
        try:
            shutil.rmtree(dir_path_ori)
        except PermissionError:
            print(f" x PermissionError! ({dir_path_ori})")


def is_dir_having_file(dir_path: str) -> bool:
    has_file = False
    for element_name in os.listdir(dir_path):
        element_path = os.path.join(dir_path, element_name)
        if os.path.isfile(element_path) and os.path.getsize(element_path) > 0:
            has_file = True
        elif os.path.isdir(element_path):
            has_file = has_file or is_dir_having_file(element_path)

        if has_file:
            break

    return has_file


def extract_work_name(
    titles: List[str],
    remove_unclosed_pair: bool = True,
    remove_tailing_slash: bool = False,
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
    tolerance_percent = 0.8  # 允许的最大次数差

    # 获取所有候选：出现次数 >= 最大次数 - 容差
    candidates = [
        (prefix, count)
        for prefix, count in prefix_counts.items()
        if count >= max_count * tolerance_percent
    ]

    # 排序规则：优先长度降序，其次次数降序，最后字典序升序
    candidates.sort(key=lambda x: (-len(x[0]), -x[1], x[0]))

    # 提取最优候选
    best_candidate = candidates[0][0] if candidates else ""

    # 后处理：移除未闭合括号及其后续内容
    return _extract_work_name_post_process(
        best_candidate,
        remove_unclosed_pair=remove_unclosed_pair,
        remove_tailing_slash=remove_tailing_slash,
    )


def _extract_work_name_post_process(
    s: str, remove_unclosed_pair: bool = True, remove_tailing_slash: bool = False
) -> str:
    """
    后处理函数：移除未闭合括号及其后续内容

    :param s: 原始字符串
    :return: 处理后的字符串
    """

    # 清除前后空格
    s = s.strip()

    if remove_unclosed_pair:
        stack = []
        last_unmatched_pos = -1

        # 遍历字符串记录括号状态
        for i, c in enumerate(s):
            if c in {"(", "["}:
                stack.append((c, i))  # 记录括号类型和位置
            elif c == ")" and stack and stack[-1][0] == "(":
                stack.pop()
            elif c == "]" and stack and stack[-1][0] == "[":
                stack.pop()

        # 如果存在未闭合括号
        if stack:
            last_unmatched_pos = stack[-1][1]
            s = s[:last_unmatched_pos].rstrip()  # 截断并移除末尾空格

    if remove_tailing_slash:
        if s.rstrip().endswith("/"):
            s = s.rstrip()[:-1].rstrip()

    return s
