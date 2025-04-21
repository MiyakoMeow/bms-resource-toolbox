import os

from bms_fs import ReplaceAction, ReplaceOptions, move_elements_across_dir


def main(src_dir: str, dst_dir: str):
    for bms_dir_name in os.listdir(src_dir):
        bms_dir = os.path.join(src_dir, bms_dir_name)
        if not os.path.isdir(bms_dir):
            continue

        print(f"Moving: {bms_dir_name}")

        dst_bms_dir = os.path.join(dst_dir, bms_dir_name)
        move_elements_across_dir(
            bms_dir,
            dst_bms_dir,
            replace_options=ReplaceOptions(
                ext=dict(
                    (ext, ReplaceAction.Replace)
                    for ext in ["ogg", "flac", "mp4", "wmv", "mpg", "mpeg", "bmp"]
                ),
                default=ReplaceAction.CheckReplace,
            ),
        )


if __name__ == "__main__":
    src_dir = input("Input the src dir:")
    dst_dir = input("Input the dst dir:")
    main(src_dir, dst_dir)
