import os
from bms_fs import move_elements_across_dir


def main(
    from_dir_path: str,
    to_dir_path: str,
    print_info: bool = False,
    replace: bool = True,
):
    move_elements_across_dir(
        from_dir_path, to_dir_path, print_info=print_info, replace=replace
    )


if __name__ == "__main__":
    src_dir = input("Input src_dir:")
    if os.path.isdir(src_dir):
        print("src_dir is not a dir.")
        exit()
    dst_dir = input("Input dst_dir:")
    if os.path.isdir(dst_dir):
        print("dst_dir is not a dir.")
        exit()
    main(src_dir, dst_dir)
