from concurrent.futures import ThreadPoolExecutor, as_completed
import multiprocessing
import os
import os.path
import shutil
import time
import zipfile

import py7zr
import rarfile

from bms_fs import get_bms_folder_dir, get_bms_pack_dir, move_files_across_dir


def unzip_file_to_cache_dir(file_path: str, cache_dir_path: str):
    file_name = file_path.rsplit("/")[-1].rsplit("\\")[-1]
    if file_path.endswith(".zip"):
        print(f"Extracting {file_path} to {cache_dir_path} (zip)")
        zip_file = zipfile.ZipFile(file_path)

        # 解压
        def unzip_single_file(
            zip_file: zipfile.ZipFile, file: zipfile.ZipInfo, cache_dir_path: str
        ):
            # 先获取原文件的时间
            d_time = file.date_time
            d_gettime = "%s/%s/%s %s:%s" % (
                d_time[0],
                d_time[1],
                d_time[2],
                d_time[3],
                d_time[4],
            )
            # 先解压文件
            zip_file.extract(file, cache_dir_path)
            # 获取解压后文件的绝对路径
            filep = os.path.join(cache_dir_path, file.filename)
            d_timearry = time.mktime(time.strptime(d_gettime, "%Y/%m/%d %H:%M"))
            # 设置解压后的修改时间(这里把修改时间与访问时间设为一样了,windows系统)
            os.utime(filep, (d_timearry, d_timearry))

        # 创建线程池
        with ThreadPoolExecutor(max_workers=multiprocessing.cpu_count()) as executor:
            # 提交任务
            futures = [
                executor.submit(unzip_single_file, zip_file, file, cache_dir_path)
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
        target_file_path = f"{cache_dir_path}/{"".join(file_name.split(" ")[1:])}"
        print(f"Coping {file_path} to {target_file_path}")
        shutil.copy(file_path, target_file_path)


def main():
    root_dir = get_bms_folder_dir()
    pack_dir = get_bms_pack_dir()

    for file_name in os.listdir(pack_dir):
        file_path = f"{pack_dir}/{file_name}"
        if not os.path.isfile(file_path):
            continue
        id_str = file_name.split(" ")[0]
        if not id_str.isdigit():
            continue

        # Create a cache dir
        cache_dir_path = f"{pack_dir}/{id_str}"
        if not os.path.isdir(cache_dir_path):
            os.mkdir(cache_dir_path)

        # Unpack & Copy
        unzip_file_to_cache_dir(file_path, cache_dir_path)

        # Unwrap dirs
        cache_dir_root_path = cache_dir_path
        cache_folder_count = 0
        cache_file_count = 0
        inner_dir_name = None
        done = False
        error = False
        while True:
            cache_folder_count = 0
            cache_file_count = 0
            inner_dir_name = None
            for cache_name in os.listdir(cache_dir_path):
                cache_path = f"{cache_dir_path}/{cache_name}"
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

            if cache_folder_count == 0:
                done = True

            if cache_folder_count == 1 and cache_file_count >= 10:
                done = True

            if cache_folder_count > 1:
                print(
                    f"{cache_dir_path}: has more then 1 folders, please do it manually."
                )
                error = True

            if done or error:
                break

            # move out files
            if inner_dir_name is not None:
                inner_dir_path = f"{cache_dir_path}/{inner_dir_name}"
                # Avoid two floor same name
                inner_inner_dir_path = os.path.join(inner_dir_path, inner_dir_name)
                if os.path.isdir(inner_inner_dir_path):
                    shutil.move(inner_inner_dir_path, f"{inner_inner_dir_path}-rep")
                # Move
                print(f"Moving inner files in {inner_dir_path} to {cache_dir_path}")
                move_files_across_dir(inner_dir_path, cache_dir_path)
                os.rmdir(inner_dir_path)

        if error:
            continue

        if cache_folder_count == 0 and cache_file_count == 0:
            print(f"{cache_dir_path}: Cache is Empty!")
            os.rmdir(cache_dir_path)
            continue

        # Find Target dir
        target_dir_path = None
        for dir_name in os.listdir(root_dir):
            dir_path = f"{root_dir}/{dir_name}"
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
            target_dir_path = f"{root_dir}/{id_str}"

        if not os.path.isdir(target_dir_path):
            os.mkdir(target_dir_path)

        # Move cache to bms dir
        print(f"Moving files in {cache_dir_path} to {target_dir_path}")
        move_files_across_dir(cache_dir_path, target_dir_path, cache_file_count <= 10)
        os.rmdir(cache_dir_path)
        try:
            os.rmdir(cache_dir_root_path)
        except FileNotFoundError:
            pass

        # Move File to Another dir
        print(f"Finish dealing with file: {file_name}")
        used_pack_dir = f"{pack_dir}/BOFTTPacks"
        if not os.path.isdir(used_pack_dir):
            os.mkdir(used_pack_dir)
        shutil.move(file_path, os.path.join(used_pack_dir, file_name))


if __name__ == "__main__":
    main()
