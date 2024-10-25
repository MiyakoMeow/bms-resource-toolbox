from concurrent.futures import Future, ThreadPoolExecutor, as_completed
import os
import subprocess
import multiprocessing
from typing import List, Optional, Tuple, Union


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
AUDIO_PRESET_WAV_FROM_FLAC_NOKEEP_METADATA = AudioPreset("flac", "wav", "-d -f")

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
) -> bool:
    """
    Example:
    wav flac flac
    wav ogg ogg -ab 320k
    """

    # 解压
    def parse_audio(
        file_path: str, preset_index: int, preset: AudioPreset
    ) -> Tuple[Tuple[str, int], Union[int, Tuple[bytes, bytes]]]:
        # New cmd
        output_file_path = (
            file_path[: -len(file_path.rsplit(".")[-1])] + preset.output_format
        )
        if os.path.isfile(output_file_path) and os.path.getsize(output_file_path) > 0:
            print(f"File {output_file_path} exists! Skipping...")
            return (file_path, preset_index), 0
        cmd = _get_audio_precess_cmd(
            file_path,
            output_file_path,
            preset,
        )
        process = subprocess.Popen(
            cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
        )
        returncode = process.wait()
        # Failed
        if returncode != 0:
            stdout, stderr = process.communicate()
            return (file_path, preset_index), (stdout, stderr)
        # Success
        if remove_origin_file and os.path.isfile(file_path):
            os.remove(file_path)
        return (file_path, preset_index), returncode

    has_error = False
    err_file_path = ""
    err_stdout = b""
    err_stderr = b""

    # 创建线程池
    hdd = True
    max_workers = (
        min(multiprocessing.cpu_count(), 24) if hdd else multiprocessing.cpu_count()
    )

    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        # 提交任务
        futures: List[
            Future[Tuple[Tuple[str, int], Union[int, Tuple[bytes, bytes]]]]
        ] = []
        # Spawn Tasks
        for file_name in os.listdir(dir):
            file_path = f"{dir}/{file_name}"
            if not os.path.isfile(file_path):
                continue
            # Check ext
            ext_found: Optional[str] = None
            for ext in input_exts:
                if file_path.lower().endswith("." + ext):
                    ext_found = ext
            if ext_found is None:
                continue
            # Submit
            future = executor.submit(parse_audio, file_path, 0, presets[0])
            futures.append(future)

        # 等待任务完成
        while len(futures) > 0:
            new_futures: List[
                Future[Tuple[Tuple[str, int], Union[int, Tuple[bytes, bytes]]]]
            ] = []
            for future in as_completed(futures):
                (file_path, preset_index), result = future.result()
                if not isinstance(result, int):
                    # Failed
                    err_file_path = file_path
                    err_stdout, err_stderr = result
                    new_preset_index = preset_index + 1
                    if new_preset_index in range(0, len(presets)):
                        # Try Next
                        new_future = executor.submit(
                            parse_audio,
                            file_path,
                            new_preset_index,
                            presets[new_preset_index],
                        )
                        new_futures.append(new_future)
                    else:
                        # Last, Return
                        has_error = True
            futures = new_futures

    if has_error:
        print("Has Error!")
        print("- Err file_path: ", err_file_path)
        print("- Err stdout: ", err_stdout)
        print("- Err stderr: ", err_stderr)

    return not has_error
