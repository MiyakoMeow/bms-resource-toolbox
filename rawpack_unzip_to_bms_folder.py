from concurrent.futures import ThreadPoolExecutor, as_completed
import multiprocessing
import os
import os.path
import shutil
import time
from typing import Dict, List
import zipfile

import py7zr
import rarfile

from bms_fs import get_bms_folder_dir, get_bms_pack_dir, move_files_across_dir


def unzip_file_to_cache_dir(file_path: str, cache_dir_path: str):
    file_name = os.path.split(file_path)[-1]
    if file_path.endswith(".zip"):
        print(f"Extracting {file_path} to {cache_dir_path} (zip)")
        zip_file = zipfile.ZipFile(file_path)

        # 解压
        zip_file.extractall(cache_dir_path)

        # 设置文件信息
        def set_file_info(file: zipfile.ZipInfo, cache_dir_path: str):
            # 先获取原文件的时间
            d_time = file.date_time
            d_gettime = "%s/%s/%s %s:%s" % (
                d_time[0],
                d_time[1],
                d_time[2],
                d_time[3],
                d_time[4],
            )
            # 获取解压后文件的绝对路径
            filep = os.path.join(cache_dir_path, file.filename)
            d_timearry = time.mktime(time.strptime(d_gettime, "%Y/%m/%d %H:%M"))
            # 设置解压后的修改时间(这里把修改时间与访问时间设为一样了,windows系统)
            os.utime(filep, (d_timearry, d_timearry))

        hdd = not file_path.upper().startswith("C:")
        max_workers = (
            min(multiprocessing.cpu_count(), 16) if hdd else multiprocessing.cpu_count()
        )
        with ThreadPoolExecutor(max_workers=max_workers) as executor:
            # 提交任务
            futures = [
                executor.submit(set_file_info, file, cache_dir_path)
                for file in zip_file.infolist()
            ]
            # 等待任务完成
            for _ in as_completed(futures):
                pass

        zip_file.close()
    elif file_path.endswith(".7z"):
        print(f"Extracting {file_path} to {cache_dir_path} (7z)")
        sevenzip_file = py7zr.SevenZipFile(file_path)
        sevenzip_file.extractall(cache_dir_path)
        sevenzip_file.close()
    elif file_path.endswith(".rar"):
        print(f"Extracting {file_path} to {cache_dir_path} (RAR)")
        rar_file = rarfile.RarFile(file_path)
        rar_file.extractall(cache_dir_path)
        rar_file.close()
    else:
        target_file_path = os.path.join(
            cache_dir_path, "".join(file_name.split(" ")[1:])
        )
        print(f"Coping {file_path} to {target_file_path}")
        shutil.copy(file_path, target_file_path)


def main(pack_dir: str, cache_dir: str, root_dir: str):
    if not os.path.isdir(cache_dir):
        os.mkdir(cache_dir)
    if not os.path.isdir(root_dir):
        os.mkdir(root_dir)
    for file_name in os.listdir(pack_dir):
        file_path = os.path.join(pack_dir, file_name)
        if not os.path.isfile(file_path):
            continue
        id_str = file_name.split(" ")[0]
        if not id_str.isdigit():
            continue

        # Create a cache dir
        cache_dir_path = os.path.join(cache_dir, id_str)
        if not os.path.isdir(cache_dir_path):
            os.mkdir(cache_dir_path)

        # Unpack & Copy
        unzip_file_to_cache_dir(file_path, cache_dir_path)

        # Unwrap dirs
        cache_dir_root_path = cache_dir_path
        cache_folder_count = 0
        cache_file_count = 0
        inner_dir_name = None
        file_ext_count: Dict[str, List[str]] = dict()
        done = False
        error = False
        while True:
            file_ext_count = dict()
            cache_folder_count = 0
            cache_file_count = 0
            inner_dir_name = None
            for cache_name in os.listdir(cache_dir_path):
                cache_path = os.path.join(cache_dir_path, cache_name)
                if os.path.isdir(cache_path):
                    # Remove __MACOSX dir
                    if cache_name == "__MACOSX":
                        shutil.rmtree(cache_path)
                        continue
                    # Normal dir
                    cache_folder_count += 1
                    inner_dir_name = cache_name
                if os.path.isfile(cache_path):
                    cache_file_count += 1
                    # Count ext
                    file_ext = file_name.rsplit(".")[-1]
                    if file_ext_count.get(file_ext) is None:
                        file_ext_count.update({file_ext: [cache_name]})
                    else:
                        file_ext_count[file_ext].append(cache_name)

            if cache_folder_count == 0:
                done = True

            if cache_folder_count == 1 and cache_file_count >= 10:
                done = True

            if cache_folder_count > 1:
                print(
                    f" !_! {cache_dir_path}: has more then 1 folders, please do it manually."
                )
                error = True

            if done or error:
                break

            # move out files
            if inner_dir_name is not None:
                inner_dir_path = os.path.join(cache_dir_path, inner_dir_name)
                # Avoid two floor same name
                inner_inner_dir_path = os.path.join(inner_dir_path, inner_dir_name)
                if os.path.isdir(inner_inner_dir_path):
                    print(f" - Renaming inner inner dir name: {inner_inner_dir_path}")
                    shutil.move(inner_inner_dir_path, f"{inner_inner_dir_path}-rep")
                # Move
                print(f" - Moving inner files in {inner_dir_path} to {cache_dir_path}")
                move_files_across_dir(inner_dir_path, cache_dir_path)
                os.rmdir(inner_dir_path)

        if error:
            continue

        if cache_folder_count == 0 and cache_file_count == 0:
            print(f" !_! {cache_dir_path}: Cache is Empty!")
            os.rmdir(cache_dir_path)
            continue

        # Has More Than 1 mp4?!
        mp4_count = file_ext_count.get("mp4")
        if mp4_count is not None and len(mp4_count) > 1:
            print(f" - Tips: {cache_dir_path} has more than 1 mp4 files!", mp4_count)

        # Find Target dir
        target_dir_path = None
        for dir_name in os.listdir(root_dir):
            dir_path = os.path.join(root_dir, dir_name)
            if not os.path.isdir(dir_path):
                continue
            if not (
                dir_name.startswith(id_str)
                and (
                    len(dir_name) == len(id_str)
                    or dir_name[len(id_str) :].startswith(".")
                )
            ):
                continue
            target_dir_path = dir_path

        if target_dir_path is None:
            target_dir_path = os.path.join(root_dir, id_str)

        if not os.path.isdir(target_dir_path):
            os.mkdir(target_dir_path)

        # Move cache to bms dir
        print(f" > Moving files in {cache_dir_path} to {target_dir_path}")
        move_files_across_dir(cache_dir_path, target_dir_path, cache_file_count <= 10)
        os.rmdir(cache_dir_path)
        try:
            os.rmdir(cache_dir_root_path)
        except FileNotFoundError:
            pass

        # Move File to Another dir
        print(f" > Finish dealing with file: {file_name}")
        used_pack_dir = os.path.join(pack_dir, "BOFTTPacks")
        if not os.path.isdir(used_pack_dir):
            os.mkdir(used_pack_dir)
        shutil.move(file_path, os.path.join(used_pack_dir, file_name))


if __name__ == "__main__":
    root_dir = get_bms_folder_dir()
    pack_dir = get_bms_pack_dir()
    main(root_dir=root_dir, pack_dir=pack_dir, cache_dir="E:\\BMSCache")
