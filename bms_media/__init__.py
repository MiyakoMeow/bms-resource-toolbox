import json
import os
import subprocess
import time
from typing import Any, Dict, List, Optional, Tuple


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
        if stream["codec_type"] != "video":
            continue
        return VideoInfo(
            int(stream["width"]),
            int(stream["height"]),
            int(stream["bit_rate"]),
        )

    return None


"""
Video
"""

VIDEO_PRESET_WMV_480P = 0
VIDEO_PRESET_WMV_512X512 = 1


def _get_video_precess_cmd(file_path: str, preset: int) -> str:
    """
    .\ffmpeg.exe -i raw_video.mkv -filter_complex "[0:v]scale=512:512:force_original_aspect_ratio=increase,crop=512:512:(ow-iw)/2:(oh-ih)/2,boxblur=20[v1];[0:v]scale=512:512:force_original_aspect_ratio=decrease[v2];[v1][v2]overlay=(main_w-overlay_w)/2:(main_h-overlay_h)/2[vid]" -map [vid] -an -c:v wmv2 -q:v 8 _bga.wmv
    """
    output_ext = "wmv"
    output_path = file_path[: -len(file_path.rsplit(".", 1)[-1])] + output_ext
    if preset == VIDEO_PRESET_WMV_480P:
        return (
            r'ffmpeg -i "'
            + file_path
            + r'" -filter_complex "[0:v]scale=640:480:force_original_aspect_ratio=increase,crop=640:480:(ow-iw)/2:(oh-ih)/2,boxblur=20[v1];[0:v]scale=640:480:force_original_aspect_ratio=decrease[v2];[v1][v2]overlay=(main_w-overlay_w)/2:(main_h-overlay_h)/2[vid]" -map [vid] -an -c:v wmv2 -q:v 8 "'
            + output_path
            + '"'
        )
    if preset == VIDEO_PRESET_WMV_512X512:
        return (
            r'ffmpeg -i "'
            + file_path
            + r'" -filter_complex "[0:v]scale=512:512:force_original_aspect_ratio=increase,crop=512:512:(ow-iw)/2:(oh-ih)/2,boxblur=20[v1];[0:v]scale=512:512:force_original_aspect_ratio=decrease[v2];[v1][v2]overlay=(main_w-overlay_w)/2:(main_h-overlay_h)/2[vid]" -map [vid] -an -c:v wmv2 -q:v 8 "'
            + output_path
            + '"'
        )
    return ""


def process_video_in_dir(
    dir: str,
    preset: int = VIDEO_PRESET_WMV_512X512,
    remove_origin_file: bool = True,
) -> bool:
    video_extes = ["mp4", "avi"]
    for file_name in os.listdir(dir):
        file_path = f"{dir}/{file_name}"
        if not os.path.isfile(file_path):
            continue
        # Check ext
        ext_checked = False
        for ext in video_extes:
            if file_name.lower().endswith("." + ext):
                ext_checked = True
        if not ext_checked:
            continue
        # Process
        cmd = _get_video_precess_cmd(file_path, preset)
        print("Process Video:", cmd)
        result = subprocess.run(
            cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
        )
        if result.returncode != 0:
            print("Stdout:", result.stdout)
            print("Stderr:", result.stderr)
            return False
        if remove_origin_file and os.path.isfile(file_path):
            os.remove(file_path)

    return True


"""
Audio
"""


class AudioPreset:
    def __init__(self, output_format: str, arg: Optional[str] = None) -> None:
        self.output_format = output_format
        self.arg = arg

    def __str__(self) -> str:
        return f"AudioPreset: output_format: {self.output_format} arg: {self.arg}"


AUDIO_PRESET_OGG = AudioPreset("ogg", None)
AUDIO_PRESET_OGG_320K = AudioPreset("ogg", "-ab 320k")
AUDIO_PRESET_WAV = AudioPreset("wav", None)
AUDIO_PRESET_FLAC = AudioPreset("flac", None)


def _get_audio_precess_cmd(
    file_path: str,
    output_file_path: str,
    audio_format: str,
    extra_args: Optional[str] = None,
) -> str:
    # Execute
    extra_args = extra_args if extra_args is not None else ""
    cmd = f'ffmpeg -hide_banner -loglevel panic -i "{file_path}" -f {audio_format} {extra_args} "{output_file_path}"'
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
    while len(processes) > 0:
        processes_waiting: List[
            Tuple[str, Optional[str], int, Optional[subprocess.Popen]]
        ] = []
        for i, (file_path, output_file_path, preset_index, process) in enumerate(
            processes
        ):
            switch_required = False
            # Empty Process?
            if 0 <= preset_index < len(presets) and process is None:
                if remove_origin_file and os.path.isfile(file_path):
                    os.remove(file_path)
                continue

            # Get status
            stdout = b""
            stderr = b""
            if process is None:
                if output_file_path is not None and os.path.isfile(output_file_path):
                    os.remove(output_file_path)
                switch_required = True
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

            # Need switch?
            if switch_required:
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
                    next_preset.output_format,
                    next_preset.arg,
                )
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
            if len(stdout) > 0:
                print(stdout)
            if len(stderr) > 0:
                print(stderr)
            # Normal End: Remove File
            if remove_origin_file and os.path.isfile(file_path):
                os.remove(file_path)

        processes = processes_waiting

        time.sleep(0.001)

    return not has_error
