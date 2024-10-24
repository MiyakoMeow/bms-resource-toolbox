import os
from typing import Dict

from bms_fs import get_bms_folder_dir


def remove_unneed_media_files(bms_dir_path: str):
    print(f"Scaning: {bms_dir_path}")
    ext_count: Dict[str, int] = dict()
    for file_name in os.listdir(bms_dir_path):
        file_path = f"{bms_dir_path}/{file_name}"
        if not os.path.isfile(file_path):
            continue

        # Count ext
        file_ext = file_name.rsplit(".")[-1]
        if ext_count.get(file_ext) is None:
            ext_count.update({file_ext: 1})
        else:
            ext_count[file_ext] += 1

        # MP4 -> WMV/MPG/MPEG
        if file_name.endswith(".mp4"):
            replacing_file_path = f"{bms_dir_path}/{file_name[:-len(".mp4")] + ".wmv"}"
            if os.path.isfile(replacing_file_path):
                print(
                    f"- Remove file {replacing_file_path}, because {file_path} exists."
                )
                os.remove(replacing_file_path)
            replacing_file_path = f"{bms_dir_path}/{file_name[:-len(".mp4")] + ".mpg"}"
            if os.path.isfile(replacing_file_path):
                print(
                    f"- Remove file {replacing_file_path}, because {file_path} exists."
                )
                os.remove(replacing_file_path)
            replacing_file_path = f"{bms_dir_path}/{file_name[:-len(".mp4")] + ".mpeg"}"
            if os.path.isfile(replacing_file_path):
                print(
                    f"- Remove file {replacing_file_path}, because {file_path} exists."
                )
                os.remove(replacing_file_path)

        # AVI -> WMV/MPG/MPEG
        if file_name.endswith(".avi"):
            replacing_file_path = f"{bms_dir_path}/{file_name[:-len(".avi")] + ".wmv"}"
            if os.path.isfile(replacing_file_path):
                print(
                    f"- Remove file {replacing_file_path}, because {file_path} exists."
                )
                os.remove(replacing_file_path)
            replacing_file_path = f"{bms_dir_path}/{file_name[:-len(".avi")] + ".mpg"}"
            if os.path.isfile(replacing_file_path):
                print(
                    f"- Remove file {replacing_file_path}, because {file_path} exists."
                )
                os.remove(replacing_file_path)
            replacing_file_path = f"{bms_dir_path}/{file_name[:-len(".mp4")] + ".mpeg"}"
            if os.path.isfile(replacing_file_path):
                print(
                    f"- Remove file {replacing_file_path}, because {file_path} exists."
                )
                os.remove(replacing_file_path)

        # FLAC -> WAV/OGG
        elif file_name.endswith(".flac"):
            replacing_file_path = f"{bms_dir_path}/{file_name[:-len(".flac")] + ".wav"}"
            if os.path.isfile(replacing_file_path):
                print(
                    f"- Remove file {replacing_file_path}, because {file_path} exists."
                )
                os.remove(replacing_file_path)
            replacing_file_path = f"{bms_dir_path}/{file_name[:-len(".flac")] + ".ogg"}"
            if os.path.isfile(replacing_file_path):
                print(
                    f"- Remove file {replacing_file_path}, because {file_path} exists."
                )
                os.remove(replacing_file_path)

        # WAV -> OGG
        elif file_name.endswith(".wav"):
            replacing_file_path = f"{bms_dir_path}/{file_name[:-len("wav")] + ".ogg"}"
            if os.path.isfile(replacing_file_path):
                print(
                    f"- Remove file {replacing_file_path}, because {file_path} exists."
                )
                os.remove(replacing_file_path)

    # Do With Ext Count
    mp4_count = ext_count.get("mp4")
    if mp4_count is not None and mp4_count > 1:
        print(f" - Tips: {bms_dir_path} has more than 1 mp4 files!")


if __name__ == "__main__":
    root_dir = get_bms_folder_dir()
    for bms_dir_name in os.listdir(root_dir):
        bms_dir_path = f"{root_dir}/{bms_dir_name}"
        if not os.path.isdir(bms_dir_path):
            continue
        remove_unneed_media_files(bms_dir_path)
