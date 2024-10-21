import copy
import json
import os
import subprocess
import multiprocessing
from typing import Any, Dict, List, Optional, Tuple

import time


"""
Media
"""


def get_media_file_probe(file_path: str) -> Dict[Any, Any]:
    cmd = (
        f'ffprobe -show_format -show_streams -print_format json -v quiet "{file_path}"'
    )
    print(f"Exec: {cmd}")
    result = subprocess.run(
        cmd,
        shell=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    out = result.stdout
    return json.loads(out)


"""
Video Info
"""


class VideoInfo:
    def __init__(self, width: int, height: int, bit_rate: int) -> None:
        self.width = width
        self.height = height
        self.bit_rate = bit_rate


def get_video_info(file_path: str) -> Optional[VideoInfo]:
    probe = get_media_file_probe(file_path)
    for stream in probe["streams"]:
        stream: Dict[Any, Any] = stream
        if stream["codec_type"] != "video":
            continue
        return VideoInfo(
            int(stream["width"]),
            int(stream["height"]),
            int(stream["bit_rate"]),
        )

    return None


def get_video_size(file_path: str) -> Optional[Tuple[int, int]]:
    probe = get_media_file_probe(file_path)
    for stream in probe["streams"]:
        stream: Dict[Any, Any] = stream
        if stream["codec_type"] != "video":
            continue
        return (
            int(stream["width"]),
            int(stream["height"]),
        )

    return None


"""
Video
"""


class VideoPreset:
    def __init__(
        self,
        exec: str,
        input_arg: Optional[str],
        fliter_arg: Optional[str],
        output_file_ext: str,
        output_codec: str,
        arg: Optional[str] = None,
    ) -> None:
        self.exec = exec
        self.input_arg = input_arg
        self.fliter_arg = fliter_arg
        self.output_file_ext = output_file_ext
        self.output_codec = output_codec
        self.arg = arg

    def __str__(self) -> str:
        return (
            f"VideoPreset {{ exec: {self.exec}, output_format: {self.output_codec} }}"
        )

    def __repr__(self) -> str:
        return self.__str__()

    def get_output_file_path(self, input_file_path: str) -> str:
        return (
            input_file_path[: -len(input_file_path.rsplit(".", 1)[-1])]
            + self.output_file_ext
        )

    def get_video_process_cmd(self, input_file_path: str, output_file_path: str) -> str:
        input_arg = self.input_arg if self.input_arg is not None else ""
        fliter_arg = self.fliter_arg if self.fliter_arg is not None else ""
        inner_arg = "-map_metadata 0" if self.exec == "ffmpeg" else ""
        return f'{self.exec} {input_arg} "{input_file_path}" {fliter_arg} {inner_arg} -c:v {self.output_codec} {self.arg} "{output_file_path}" '


FLITER_512X512 = '-filter_complex "[0:v]scale=512:512:force_original_aspect_ratio=increase,crop=512:512:(ow-iw)/2:(oh-ih)/2,boxblur=20[v1];[0:v]scale=512:512:force_original_aspect_ratio=decrease[v2];[v1][v2]overlay=(main_w-overlay_w)/2:(main_h-overlay_h)/2[vid]" -map [vid]'
FLITER_480P = '-filter_complex "[0:v]scale=640:480:force_original_aspect_ratio=increase,crop=640:480:(ow-iw)/2:(oh-ih)/2,boxblur=20[v1];[0:v]scale=640:480:force_original_aspect_ratio=decrease[v2];[v1][v2]overlay=(main_w-overlay_w)/2:(main_h-overlay_h)/2[vid]" -map [vid]'

VIDEO_PRESET_MPEG1VIDEO_480P = VideoPreset(
    "ffmpeg", "-hide_banner -i", FLITER_480P, "mpg", "mpeg1video", "-an -b:v 1500k"
)
VIDEO_PRESET_MPEG1VIDEO_512X512 = VideoPreset(
    "ffmpeg", "-hide_banner -i", FLITER_512X512, "mpg", "mpeg1video", "-an -b:v 1500k"
)
VIDEO_PRESET_WMV1_480P = VideoPreset(
    "ffmpeg", "-hide_banner -i", FLITER_480P, "wmv", "wmv1", "-an -b:v 1500k"
)
VIDEO_PRESET_WMV1_512X512 = VideoPreset(
    "ffmpeg", "-hide_banner -i", FLITER_512X512, "wmv", "wmv1", "-an -b:v 1500k"
)
VIDEO_PRESET_WMV2_480P = VideoPreset(
    "ffmpeg", "-hide_banner -i", FLITER_480P, "wmv", "wmv2", "-an -b:v 1500k"
)
VIDEO_PRESET_WMV2_512X512 = VideoPreset(
    "ffmpeg", "-hide_banner -i", FLITER_512X512, "wmv", "wmv2", "-an -b:v 1500k"
)


def get_prefered_preset_list(file_path: str) -> List[VideoPreset]:
    video_size = get_video_size(file_path)
    if video_size is None:
        return []
    width, height = video_size
    if width / height > 640 / 480:
        return [VIDEO_PRESET_MPEG1VIDEO_480P, VIDEO_PRESET_WMV1_480P]
    else:
        return [VIDEO_PRESET_MPEG1VIDEO_512X512, VIDEO_PRESET_WMV1_512X512]


def process_video_in_dir(
    dir: str,
    input_exts: List[str] = ["mp4", "avi"],
    presets: List[VideoPreset] = [VIDEO_PRESET_MPEG1VIDEO_480P, VIDEO_PRESET_WMV1_480P],
    remove_origin_file: bool = True,
    use_prefered: bool = False,
) -> bool:
    has_error = False
    for file_name in os.listdir(dir):
        file_path = f"{dir}/{file_name}"
        if not os.path.isfile(file_path):
            continue
        # Check ext
        ext_checked = False
        for ext in input_exts:
            if file_name.lower().endswith("." + ext):
                ext_checked = True
        if not ext_checked:
            continue
        # Get prefered
        presets_for_file = copy.deepcopy(presets)
        if use_prefered:
            presets_new = get_prefered_preset_list(file_path)
            presets_new.extend(presets_for_file)
            presets_for_file = presets_new
        # Check With Presets:
        for preset in presets_for_file:
            output_file_path = preset.get_output_file_path(file_path)
            if file_path == output_file_path:
                # This file is the preset's output.
                break
            if os.path.isfile(output_file_path):
                print(f"File exists: {output_file_path}")
                continue
            # Process
            cmd = preset.get_video_process_cmd(file_path, output_file_path)
            print(f"Processing Video: {file_path} Preset: {preset}")
            result = subprocess.run(
                cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
            )
            if result.returncode != 0:
                if os.path.isfile(output_file_path):
                    os.remove(output_file_path)
                if preset is presets[-1]:
                    print("Has Error!")
                    print("Cmd: ", cmd)
                    print("Stdout:", result.stdout)
                    print("Stderr:", result.stderr)
                    has_error = True
                    break
                continue

            # Normal End: Success
            if remove_origin_file and os.path.isfile(file_path):
                os.remove(file_path)
            # No need to exec next ctrl.
            break

    return not has_error


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
AUDIO_PRESET_WAV_FROM_FLAC = AudioPreset("flac", "wav", "-d --keep-foreign-metadata")
AUDIO_PRESET_FLAC = AudioPreset("ffmpeg", "flac", "--keep-foreign-metadata --best")


def _get_audio_precess_cmd(
    file_path: str,
    output_file_path: str,
    preset: AudioPreset,
) -> str:
    # Execute
    arg = preset.arg if preset.arg is not None else ""
    cmd = ""
    if preset.exec == "ffmpeg":
        cmd = f'ffmpeg -hide_banner -loglevel panic -i "{file_path}" -f {preset.output_format} -map_metadata 0 {arg} "{output_file_path}"'
    elif preset.exec == "oggenc":
        cmd = f'oggenc {arg} "{file_path}" -o "{output_file_path}"'
    elif preset.exec == "flac":
        cmd = f'flac {arg} "{file_path}" -o "{output_file_path}"'
    return cmd


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
            max_process_count = os.cpu_count()
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
