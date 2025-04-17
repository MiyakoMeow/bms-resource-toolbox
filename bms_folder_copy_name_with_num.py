import os
import shutil


def main(src_dir: str = "", dst_dir: str = ""):
    """
    该脚本使用于以下情况：
    已经有一个文件夹A，它的子文件夹名为“”等带有编号+小数点的形式。
    现在有另一个文件夹B，它的子文件夹名都只有编号。
    将A中的子文件夹名，同步给B的对应的子文件夹。
    """
    if len(src_dir) == 0:
        src_dir = input("Input Src Dir:")
    if len(dst_dir) == 0:
        dst_dir = input("Input Dst Dir:")
    src_dir_names = [
        dir_name
        for dir_name in os.listdir(src_dir)
        if os.path.isdir(os.path.join(src_dir, dir_name))
    ]
    # List Dst Dir
    for dir_name in os.listdir(dst_dir):
        dir_path = os.path.join(dst_dir, dir_name)
        # Get Num
        dir_num = dir_name.split(" ")[0].split(".")[0]
        if not dir_num.isdigit():
            continue
        # Search src name
        for src_name in src_dir_names:
            if not src_name.startswith(dir_num):
                continue
            # Rename
            target_dir_path = os.path.join(dst_dir, src_name)
            print(f"Rename {dir_name} to {src_name}")
            shutil.move(dir_path, target_dir_path)
            break


if __name__ == "__main__":
    main()
