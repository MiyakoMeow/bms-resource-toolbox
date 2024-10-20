import os
import shutil
from typing import Optional


def _sync(
    src_dir: str,
    dst_dir: str,
    allow_exts: list[str],
    disallow_exts: list[str],
    allow_others: bool,
):
    src_list = os.listdir(src_dir)
    dst_list = os.listdir(dst_dir)
    for dst_element in src_list:
        src_path = f"{src_dir}/{dst_element}"
        dst_path = f"{dst_dir}/{dst_element}"
        if os.path.isdir(src_path):
            # Src: Dir
            if os.path.isdir(dst_path):
                _sync(src_path, dst_path, allow_exts, disallow_exts, allow_others)
            else:
                os.mkdir(dst_path)
                _sync(src_path, dst_path, allow_exts, disallow_exts, allow_others)
        elif os.path.isfile(src_path):
            # Src: File
            # Check modify time
            if not os.path.isfile(dst_path) or os.path.getmtime(
                src_path
            ) != os.path.getmtime(dst_path):
                print(f"Src Round: Copy {src_path} to {dst_path}")
                shutil.copy(src_path, dst_path)

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
    _sync(src_dir, dst_dir, [], [], True)


if __name__ == "__main__":
    main()
