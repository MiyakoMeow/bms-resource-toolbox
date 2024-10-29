import os
import subprocess
import time
import multiprocessing
from typing import List, Optional, Tuple


"""
Audio
"""


class AudioPreset:
    def __init__(
        self, exec: str, output_format: str, arg: Optional[str] = None
    ) -> None:
        self.exec = exec
        self.output_format = output_format
        self.arg = arg

    def __str__(self) -> str:
        return f"AudioPreset {{ exec: {self.exec}, output_format: {self.output_format} arg: {self.arg} }}"

    def __repr__(self) -> str:
        return self.__str__()


AUDIO_PRESET_OGG_Q10 = AudioPreset("oggenc", "ogg", "-q10")
AUDIO_PRESET_OGG_FFMPEG = AudioPreset("ffmpeg", "ogg", "")

AUDIO_PRESET_WAV_FFMPEG = AudioPreset("ffmpeg", "wav", None)
AUDIO_PRESET_WAV_FROM_FLAC = AudioPreset(
    "flac", "wav", "-d --keep-foreign-metadata-if-present -f"
)

AUDIO_PRESET_FLAC = AudioPreset(
    "flac", "flac", "--keep-foreign-metadata-if-present --best -f"
)
AUDIO_PRESET_FLAC_FFMPEG = AudioPreset("ffmpeg", "flac", "")


def _get_audio_precess_cmd(
    file_path: str,
    output_file_path: str,
    preset: AudioPreset,
) -> str:
    # Execute
    arg = preset.arg if preset.arg is not None else ""
    if preset.exec == "ffmpeg":
        return f'ffmpeg -hide_banner -loglevel panic -i "{file_path}" -f {preset.output_format} -map_metadata 0 {arg} "{output_file_path}"'
    elif preset.exec == "oggenc":
        return f'oggenc {arg} "{file_path}" -o "{output_file_path}"'
    elif preset.exec == "flac":
        return f'flac {arg} "{file_path}" -o "{output_file_path}"'
    else:
        return ""


def transfer_audio_by_format_in_dir(
    dir: str,
    input_exts: List[str],
    presets: List[AudioPreset],
    remove_origin_file: bool = True,
    remove_existing_target_file: bool = True,
) -> bool:
    """
    Example:
    wav flac flac
    wav ogg ogg -ab 320k
    """

    def check_input_file(
        dir: str, file_name: str, input_exts: List[str]
    ) -> Optional[str]:
        file_path = f"{dir}/{file_name}"
        if not os.path.isfile(file_path):
            return None
        # Check ext
        ext_found: Optional[str] = None
        for ext in input_exts:
            if file_path.lower().endswith("." + ext):
                ext_found = ext
        if ext_found is None:
            return None
        return os.path.join(dir, file_name)

    def spawn_parse_audio_process(
        file_path: str, preset_index: int, preset: AudioPreset
    ) -> Tuple[Tuple[str, int], Optional[subprocess.Popen]]:
        # New cmd
        output_file_path = (
            file_path[: -len(file_path.rsplit(".")[-1])] + preset.output_format
        )
        # Target File exists?
        if os.path.isfile(output_file_path):
            if (
                os.path.getsize(output_file_path) > 0
                and not remove_existing_target_file
            ):
                print(f" - File {output_file_path} exists! Skipping...")
                return (file_path, preset_index), None
            else:
                print(f" - Remove existing file: {output_file_path}")
                os.remove(output_file_path)
        # Run cmd
        cmd = _get_audio_precess_cmd(
            file_path,
            output_file_path,
            preset,
        )
        if len(cmd) == 0:
            return (file_path, preset_index), None
        process = subprocess.Popen(
            cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
        )
        return (file_path, preset_index), process

    has_error = False
    err_file_path = ""
    err_stdout = b""
    err_stderr = b""

    # 创建线程池
    hdd = not dir.upper().startswith("C:")
    max_workers = (
        min(multiprocessing.cpu_count(), 24) if hdd else multiprocessing.cpu_count()
    )

    # Submit
    processes: List[Tuple[Tuple[str, int], Optional[subprocess.Popen]]] = []
    task_args: List[Tuple[str, int, AudioPreset]] = [
        (os.path.join(dir, file_name), 0, presets[0])
        for file_name in os.listdir(dir)
        if check_input_file(dir, file_name, input_exts) is not None
    ]

    # Count
    file_count = len(task_args)
    fallback_files: List[Tuple[str, int]] = []

    for task_arg in task_args:
        if len(processes) >= max_workers:
            break
        processes.append(spawn_parse_audio_process(*task_arg))
    task_args = task_args[len(processes) :]

    # 等待所有任务完成
    while len(processes) > 0:
        new_processes: List[Tuple[Tuple[str, int], Optional[subprocess.Popen]]] = []

        # 检查进程状态
        switch_next_list: List[Tuple[str, int]] = []
        for process in processes:
            (file_path, preset_index), process = process
            # Switch Next?
            switch_next = False
            if process is None:
                # Empty process
                switch_next = True
            else:
                process_returncode = process.poll()
                if process_returncode is None:
                    # Running
                    new_processes.append(((file_path, preset_index), process))
                elif process_returncode == 0:
                    # Succcess
                    if remove_origin_file and os.path.isfile(file_path):
                        os.remove(file_path)
                else:
                    # Failed
                    switch_next = True
                    err_stdout, err_stderr = process.communicate()

            if switch_next:
                switch_next_list.append((file_path, preset_index))

        # 切换下个预设
        for file_path, preset_index in switch_next_list:
            new_preset_index = preset_index + 1
            if new_preset_index not in range(0, len(presets)):
                # Last, Return
                has_error = True
                continue
            # Count
            fallback_files.append((file_path, new_preset_index))
            # Try Next
            task_args.append((file_path, new_preset_index, presets[new_preset_index]))

        # 启动新进程
        running_count_delta = 0
        for task_arg in task_args:
            if len(new_processes) >= max_workers:
                break
            new_processes.append(spawn_parse_audio_process(*task_arg))
            running_count_delta += 1
        task_args = task_args[running_count_delta:]

        processes = new_processes

        # 休眠一阵子
        time.sleep(0.001)

    if has_error:
        print("Has Error!")
        print("- Err file_path: ", err_file_path)
        print("- Err stdout: ", err_stdout)
        print("- Err stderr: ", err_stderr)

    if file_count > 0:
        print(f" -v- Parsed {file_count} file(s).")
    if len(fallback_files) > 0:
        print(f" x_x Fallback: {fallback_files}.")

    return not has_error
