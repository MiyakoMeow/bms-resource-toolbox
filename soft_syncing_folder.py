import os
import shutil
from typing import Optional


def _sync(
    src_dir: str,
    dst_dir: str,
    allow_src_exts: list[str],
    disallow_src_exts: list[str],
    allow_others: bool,
    remove_dst_files: bool,
):
    src_list = os.listdir(src_dir)
    dst_list = os.listdir(dst_dir)
    for dst_element in src_list:
        src_path = f"{src_dir}/{dst_element}"
        dst_path = f"{dst_dir}/{dst_element}"
        if os.path.isdir(src_path):
            # Src: Dir
            if os.path.isdir(dst_path):
                _sync(
                    src_path,
                    dst_path,
                    allow_src_exts,
                    disallow_src_exts,
                    allow_others,
                    remove_dst_files,
                )
            else:
                os.mkdir(dst_path)
                _sync(
                    src_path,
                    dst_path,
                    allow_src_exts,
                    disallow_src_exts,
                    allow_others,
                    remove_dst_files,
                )
        elif os.path.isfile(src_path):
            # Src: File
            # Check Ext
            ext_check_passed = allow_others
            ext = dst_element.rsplit(".")[-1]
            if ext in allow_src_exts:
                ext_check_passed = True
            if ext in disallow_src_exts:
                ext_check_passed = False
            if not ext_check_passed:
                continue
            # Check modify time
            src_mtime = os.path.getmtime(src_path)
            src_size = os.path.getsize(src_path)
            if (
                not os.path.isfile(dst_path)
                or src_size != os.path.getsize(dst_path)
                or src_mtime != os.path.getmtime(dst_path)
            ):
                print(f"Src Round: Copy {src_path} to {dst_path}")
                shutil.copy(src_path, dst_path)
                os.utime(dst_path, (src_mtime, src_mtime))

    if not remove_dst_files:
        return

    for dst_element in dst_list:
        src_path = f"{src_dir}/{dst_element}"
        dst_path = f"{dst_dir}/{dst_element}"
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


SYNC_PRESET_DEFAULT = [[], [], True, True]
SYNC_PRESET_FLAC = [["flac"], [], False, False]
SYNC_PRESET_MP4_AVI = [["mp4", "avi"], [], False, False]
SYNC_PRESET_CACHE = [["mp4", "avi", "flac"], [], False, False]

SYNC_PRESETS = [
    SYNC_PRESET_DEFAULT,
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
    selection_str = input("Input Selection (default=0):").strip()
    selection = 0
    if len(selection_str) > 0:
        selection = int(selection_str)

    _sync(src_dir, dst_dir, *SYNC_PRESETS[selection])


if __name__ == "__main__":
    main()
