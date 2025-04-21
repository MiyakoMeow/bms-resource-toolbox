import os

import bigpack_folder_move


def main():
    src_dir = input("Input the src dir:")
    if src_dir.startswith('"') and src_dir.endswith('"'):
        src_dir = src_dir[1:-1]
    if not os.path.isdir(src_dir):
        print(f"{src_dir}: not a dir.")
        return
    dirs = [
        dir for dir in os.listdir(src_dir) if os.path.isdir(os.path.join(src_dir, dir))
    ]
    for dir in dirs:
        if dir.find("Aery") == -1 and dir.find("AERY") == -1:
            continue
        dir_path = os.path.join(src_dir, dir)
        for i in range(len(dir)):
            sub_len = i + 1
            scan_dirs = [
                sub_dir
                for sub_dir in dirs
                if sub_dir.startswith(dir[:sub_len]) and sub_dir != dir
            ]
            if len(scan_dirs) != 1:
                continue
            scan_dir = scan_dirs[0]
            scan_dir_path = os.path.join(src_dir, scan_dir)
            print(f"Moving: {dir_path} => {scan_dir_path}")
            bigpack_folder_move.main(dir_path, scan_dir_path)
            break


if __name__ == "__main__":
    main()
