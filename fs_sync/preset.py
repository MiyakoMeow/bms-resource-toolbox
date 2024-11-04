from enum import Enum
import hashlib
import os
from typing import List, Tuple


class SoftSyncExec(Enum):
    NONE = 0
    COPY = 1
    MOVE = 2

    def __str__(self) -> str:
        match self:
            case SoftSyncExec.NONE:
                return "无操作"
            case SoftSyncExec.COPY:
                return "使用复制命令"
            case SoftSyncExec.MOVE:
                return "使用移动命令"

    def __repr__(self) -> str:
        return self.__str__()


class SoftSyncPreset:
    def __init__(
        self,
        name: str = "本地文件同步预设",
        allow_src_exts: List[str] = [],
        disallow_src_exts: List[str] = [],
        allow_other_exts: bool = True,
        no_activate_ext_bound_pairs: List[Tuple[List[str], List[str]]] = [],
        remove_dst_extra_files: bool = True,
        check_file_size: bool = True,
        check_file_mtime: bool = True,
        check_file_sha512: bool = False,
        remove_src_same_files: bool = False,
        exec: SoftSyncExec = SoftSyncExec.COPY,
    ) -> None:
        self.name = name
        self.allow_src_exts = allow_src_exts
        self.disallow_src_exts = disallow_src_exts
        self.allow_other_exts = allow_other_exts
        self.remove_dst_extra_files = remove_dst_extra_files
        self.no_activate_ext_bound_pairs = no_activate_ext_bound_pairs
        self.check_file_size = check_file_size
        self.check_file_mtime = check_file_mtime
        self.check_file_sha512 = check_file_sha512
        self.remove_src_same_files = remove_src_same_files
        self.exec = exec

    def __str__(self) -> str:
        ret = f"{self.name}："
        # Copy/Move
        ret += f"{self.exec}"
        ret += " "
        # Ext
        if self.allow_other_exts:
            ret += "允许同步未定义扩展名"
            ret += " "
        if len(self.allow_src_exts):
            ret += f"允许扩展名：{self.allow_src_exts}"
            ret += " "
        if len(self.disallow_src_exts):
            ret += f"拒绝扩展名：{self.disallow_src_exts}"
            ret += " "
        # Remove Src
        if self.remove_src_same_files:
            ret += "移除源中相对于目标，不需要同步的文件"
            ret += " "
        # Remove Dst
        if self.remove_dst_extra_files:
            ret += "移除目标文件夹相对源文件夹的多余文件"
            ret += " "
        # Check
        if self.check_file_mtime:
            ret += "检查修改时间"
            ret += " "
        if self.check_file_size:
            ret += "检查大小"
            ret += " "
        if self.check_file_sha512:
            ret += "检查SHA-512"
            ret += " "
        return ret


def get_file_sha512(file_path: str) -> str:
    if not os.path.isfile(file_path):
        # print(f"{file_path}：文件不存在。")
        return ""
    h = hashlib.sha512()
    with open(file_path, "rb") as f:
        b = f.read()
        h.update(b)
    ret = h.hexdigest()
    # print(f" - {file_path}: {ret}")
    return ret


SYNC_PRESET_DEFAULT = SoftSyncPreset()
SYNC_PRESET_FOR_APPEND = SoftSyncPreset(
    name="同步预设（用于更新包）",
    check_file_size=True,
    check_file_mtime=False,
    check_file_sha512=True,
    remove_src_same_files=True,
    remove_dst_extra_files=False,
    exec=SoftSyncExec.NONE,
)
SYNC_PRESET_FLAC = SoftSyncPreset(
    allow_src_exts=["flac"], allow_other_exts=False, remove_dst_extra_files=False
)
SYNC_PRESET_MP4_AVI = SoftSyncPreset(
    allow_src_exts=["mp4", "avi"], allow_other_exts=False, remove_dst_extra_files=False
)
SYNC_PRESET_CACHE = SoftSyncPreset(
    allow_src_exts=["mp4", "avi", "flac"],
    allow_other_exts=False,
    remove_dst_extra_files=False,
    exec=SoftSyncExec.NONE,
)

SYNC_PRESETS: List[SoftSyncPreset] = [
    SYNC_PRESET_DEFAULT,
    SYNC_PRESET_FOR_APPEND,
    SYNC_PRESET_FLAC,
    SYNC_PRESET_MP4_AVI,
    SYNC_PRESET_CACHE,
]
