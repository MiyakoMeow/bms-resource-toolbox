import os

from bms_fs import get_bms_folder_dir


def is_dir_empty(dir_path: str) -> bool:
    return len(os.listdir(dir_path)) == 0


def main(bms_dir: str = ""):
    if len(bms_dir) == 0:
        bms_dir = get_bms_folder_dir()

    for dir_name in os.listdir(bms_dir):
        dir_path = os.path.join(bms_dir, dir_name)
        if not os.path.isdir(dir_path):
            continue
        if is_dir_empty(dir_path):
            print(f"Remove empty dir: {dir_path}")
            os.rmdir(dir_path)


if __name__ == "__main__":
    main()
