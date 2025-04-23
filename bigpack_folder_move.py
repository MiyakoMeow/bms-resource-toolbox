import os

from bms_fs import (
    REPLACE_OPTION_UPDATE_PACK,
    move_elements_across_dir,
)


def main(src_dir: str, dst_dir: str):
    if src_dir == dst_dir:
        return
    move_count = 0
    for bms_dir_name in os.listdir(src_dir):
        bms_dir = os.path.join(src_dir, bms_dir_name)
        if not os.path.isdir(bms_dir):
            continue

        print(f"Moving: {bms_dir_name}")

        dst_bms_dir = os.path.join(dst_dir, bms_dir_name)
        move_elements_across_dir(
            bms_dir,
            dst_bms_dir,
            replace_options=REPLACE_OPTION_UPDATE_PACK,
        )
        move_count += 1
    if move_count > 0:
        print(f"Move {move_count} songs.")
        return

    # Deal with song dir
    move_elements_across_dir(
        src_dir,
        dst_dir,
        replace_options=REPLACE_OPTION_UPDATE_PACK,
    )


if __name__ == "__main__":
    src_dir = input("Input the src dir:")
    if src_dir.startswith('"') and src_dir.endswith('"'):
        src_dir = src_dir[1:-1]
    dst_dir = input("Input the dst dir:")
    if dst_dir.startswith('"') and dst_dir.endswith('"'):
        dst_dir = dst_dir[1:-1]
    main(src_dir, dst_dir)
