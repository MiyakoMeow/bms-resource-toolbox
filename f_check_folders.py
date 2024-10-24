import os

from bms_fs import get_bms_folder_dir


def main():
    bms_dir = get_bms_folder_dir()

    max_count = 483

    for no in range(1, max_count + 1):
        folder_path = os.path.join(bms_dir, str(no))
        if not os.path.isdir(folder_path):
            print(f"{folder_path} is not exist!")


if __name__ == "__main__":
    main()
