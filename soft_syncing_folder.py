from enum import Enum
import hashlib
import os
import shutil
from typing import List, Optional, Tuple


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


def _sync(
    src_dir: str,
    dst_dir: str,
    preset: SoftSyncPreset,
):
    src_list = os.listdir(src_dir)
    dst_list = os.listdir(dst_dir)
    for src_element in src_list:
        src_path = os.path.join(src_dir, src_element)
        dst_path = os.path.join(dst_dir, src_element)
        if os.path.isdir(src_path):
            # Src: Dir
            if os.path.isdir(dst_path):
                _sync(
                    src_path,
                    dst_path,
                    preset,
                )
            else:
                os.mkdir(dst_path)
                _sync(
                    src_path,
                    dst_path,
                    preset,
                )
        elif os.path.isfile(src_path):
            # Src: File
            # Check Ext
            ext_check_passed = preset.allow_other_exts
            ext = src_element.rsplit(".")[-1]
            if ext in preset.allow_src_exts:
                ext_check_passed = True
            if ext in preset.disallow_src_exts:
                ext_check_passed = False
            if not ext_check_passed:
                continue
            # Check Ext Bound
            ext_in_bound = False
            for (
                ext_bound_from_list,
                ext_bound_to_list,
            ) in preset.no_activate_ext_bound_pairs:
                if ext not in ext_bound_from_list:
                    continue
                # Found: Bound From
                for ext_bound_to in ext_bound_to_list:
                    bound_file_path = dst_path[: -len(ext)] + ext_bound_to
                    if not os.path.isfile(bound_file_path):
                        continue
                    # Found: Bound To
                    ext_in_bound = True
                    break
                if ext_in_bound:
                    break
            if ext_in_bound:
                continue
            # Replace: Check
            dst_file_exists = os.path.isfile(dst_path)
            is_same_file = True
            if is_same_file and dst_file_exists and preset.check_file_size:
                src_size = os.path.getsize(src_path)
                dst_size = os.path.getsize(dst_path)
                if src_size != dst_size:
                    print(f"{src_element}: Size Differ!")
                is_same_file = is_same_file and src_size == dst_size
            if is_same_file and dst_file_exists and preset.check_file_mtime:
                src_mtime = os.path.getmtime(src_path)
                dst_mtime = os.path.getmtime(dst_path)
                if src_mtime != dst_mtime:
                    print(f"{src_element}: MTime Differ!")
                is_same_file = is_same_file and src_mtime == dst_mtime
            if is_same_file and dst_file_exists and preset.check_file_sha512:
                src_value = get_file_sha512(src_path)
                dst_value = get_file_sha512(dst_path)
                if src_value != dst_value:
                    print(f"{src_element}: Hash Differ!")
                is_same_file = is_same_file and src_value == dst_value
            # Replace: Exec
            if not dst_file_exists or not is_same_file:
                src_mtime = os.path.getmtime(src_path)
                match preset.exec:
                    case SoftSyncExec.NONE:
                        pass
                    case SoftSyncExec.COPY:
                        print(f"Src Round: Copy {src_path} to {dst_path}")
                        shutil.copy(src_path, dst_path)
                        # Set atime/mtime
                        os.utime(dst_path, (src_mtime, src_mtime))
                    case SoftSyncExec.MOVE:
                        print(f"Src Round: Move {src_path} to {dst_path}")
                        shutil.move(src_path, dst_path)
                        # Set atime/mtime
                        os.utime(dst_path, (src_mtime, src_mtime))
            # Remove same ori files
            if (
                dst_file_exists
                and is_same_file
                and os.path.isfile(src_path)
                and preset.remove_src_same_files
            ):
                print(f"Src Round: RMFILE {src_path}")
                os.remove(src_path)

    if not preset.remove_dst_extra_files:
        return

    for dst_element in dst_list:
        src_path = os.path.join(src_dir, dst_element)
        dst_path = os.path.join(dst_dir, dst_element)
        if os.path.isdir(dst_path):
            if os.path.isdir(src_path):
                pass
            else:
                print(f"Dst Round: RMDIR: {dst_path}")
                os.rmdir(dst_path)
        elif os.path.isfile(dst_path):
            if not os.path.isfile(src_path):
                print(f"Dst Round: RMFILE: {dst_path}")
                os.remove(dst_path)


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


def main(
    src_dir: Optional[str] = None,
    dst_dir: Optional[str] = None,
):
    if src_dir is None:
        src_dir = input("Input Src Dir:")
    if dst_dir is None:
        dst_dir = input("Input Dst Dir:")
    if not os.path.isdir(src_dir):
        print("Src dir is not a dir!")
        return
    if not os.path.isdir(dst_dir):
        print("Dst dir is not a dir!")
        return
    if src_dir.strip() == dst_dir.strip():
        print("Src dir and Dst dir is same!")
        return
    # Select Preset
    print("Sync selections: ")
    for i, preset in enumerate(SYNC_PRESETS):
        print(f"  {i} - {preset}")

    while True:
        selection_str = input("Input Selection (default=0):").strip()
        selection = 0
        if len(selection_str) > 0:
            selection = int(selection_str)
            break

    _sync(src_dir, dst_dir, SYNC_PRESETS[selection])


if __name__ == "__main__":
    main()
