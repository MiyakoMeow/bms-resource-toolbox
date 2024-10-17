import os

from bms_fs import get_bms_folder_dir


def remove_unneed_media_files(bms_dir_path: str):
    print(f"Scaning: {bms_dir_path}")
    for file_name in os.listdir(bms_dir_path):
        file_path = f"{bms_dir_path}/{file_name}"
        if not os.path.isfile(file_path):
            continue

        # MP4 -> WMV/MPG
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

        # AVI -> WMV/MPG
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


if __name__ == "__main__":
    root_dir = get_bms_folder_dir()
    for bms_dir_name in os.listdir(root_dir):
        bms_dir_path = f"{root_dir}/{bms_dir_name}"
        if not os.path.isdir(bms_dir_path):
            continue
        remove_unneed_media_files(bms_dir_path)
