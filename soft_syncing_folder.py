import os
from typing import Optional

from fs.sync import sync_folder, SYNC_PRESETS


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

    sync_folder(src_dir, dst_dir, SYNC_PRESETS[selection])


if __name__ == "__main__":
    main()
