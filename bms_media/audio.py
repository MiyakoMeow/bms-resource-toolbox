import os
import subprocess
import multiprocessing
import time
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
AUDIO_PRESET_WAV = AudioPreset("ffmpeg", "wav", None)
AUDIO_PRESET_WAV_FROM_FLAC = AudioPreset(
    "flac", "wav", "-d --keep-foreign-metadata-if-present -f"
)
AUDIO_PRESET_FLAC = AudioPreset(
    "flac", "flac", "--keep-foreign-metadata-if-present --best -f"
)


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
) -> bool:
    """
    Example:
    wav flac flac
    wav ogg ogg -ab 320k
    """
    # Spawn Tasks
    # (file, output_file, running_preset_index, precess)
    processes: List[Tuple[str, Optional[str], int, Optional[subprocess.Popen]]] = []
    for file_name in os.listdir(dir):
        file_path = f"{dir}/{file_name}"
        if not os.path.isfile(file_path):
            continue
        processes.append((file_path, None, -1, None))

    # Check Tasks
    has_error = False
    err_err_example = b""
    err_out_example = b""
    while len(processes) > 0:
        processes_waiting: List[
            Tuple[str, Optional[str], int, Optional[subprocess.Popen]]
        ] = []
        for file_path, output_file_path, preset_index, process in processes:
            switch_required = False
            # Empty Process?
            if 0 <= preset_index < len(presets) and process is None:
                if remove_origin_file and os.path.isfile(file_path):
                    os.remove(file_path)
                continue

            # Process count limit
            hdd = True
            max_process_count = 2 if hdd else os.cpu_count()
            if max_process_count is None:
                max_process_count = multiprocessing.cpu_count()
            now_process_count = 0

            # Get status
            stdout = b""
            stderr = b""
            if process is None:
                # Empty Process?
                if output_file_path is not None and os.path.isfile(output_file_path):
                    os.remove(output_file_path)
                switch_required = True
                # Decrease process count
                now_process_count -= 1
            else:
                stdout, stderr = process.communicate()
                # Not Finished?
                poll_result = process.poll()
                if poll_result is None:
                    processes_waiting.append(
                        (file_path, output_file_path, preset_index, process)
                    )
                    continue
                # Has Error?
                if poll_result != 0:
                    if output_file_path is not None and os.path.isfile(
                        output_file_path
                    ):
                        os.remove(output_file_path)
                    switch_required = True
                    err_err_example = stderr
                    err_out_example = stdout
                # Finished
                # Decrease process count
                now_process_count -= 1
                # No error? Do nothing, go next.

            # Need switch?
            if switch_required:
                # check process count limit
                if now_process_count >= max_process_count:
                    processes_waiting.append(
                        (file_path, output_file_path, preset_index, process)
                    )
                    continue
                # Check ext
                ext_found: Optional[str] = None
                for ext in input_exts:
                    if file_path.lower().endswith("." + ext):
                        ext_found = ext
                if ext_found is None:
                    continue
                # get preset
                next_preset_index = preset_index + 1
                if next_preset_index >= len(presets):
                    has_error = True
                    continue
                next_preset = presets[next_preset_index]
                # New cmd
                new_output_file_path = (
                    file_path[: -len(ext_found)] + next_preset.output_format
                )
                if os.path.isfile(new_output_file_path):
                    print(f"File {new_output_file_path} exists! Skipping...")
                    continue
                cmd = _get_audio_precess_cmd(
                    file_path,
                    new_output_file_path,
                    next_preset,
                )
                # print("Audio Process: Exec:", cmd)
                # count process
                now_process_count += 1
                if len(cmd) == 0:
                    processes_waiting.append(
                        (file_path, new_output_file_path, next_preset_index, None)
                    )
                    continue
                process = subprocess.Popen(
                    cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
                )
                processes_waiting.append(
                    (file_path, new_output_file_path, next_preset_index, process)
                )
                continue

            # Normal End: Print
            # print(f" - Finished: {file_path} -> {i}")
            # Normal End: Remove File
            if remove_origin_file and os.path.isfile(file_path):
                os.remove(file_path)

        processes = processes_waiting

        time.sleep(0.000_001)

    if len(err_err_example) > 0:
        print(f"Err: {err_err_example}")
    if len(err_out_example) > 0:
        print(f"ErrOut: {err_out_example}")
    return not has_error
