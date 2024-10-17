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


def _get_audio_precess_cmd(
    file_path: str,
    input_ext: str,
    output_ext: str,
    audio_format: str,
    extra_args: Optional[str] = None,
) -> str:
    output_path = file_path[: -len(input_ext)] + output_ext
    if os.path.isfile(output_path):
        print(f"File {output_path} exists! Skipping...")
        return ""
    # Execute
    extra_args = extra_args if extra_args is not None else ""
    cmd = f'ffmpeg -hide_banner -loglevel panic -i "{file_path}" -f {audio_format} {extra_args} "{output_path}"'
    return cmd


def transfer_audio_by_format_in_dir(
    dir: str,
    input_ext: str,
    output_ext: str,
    audio_format: str,
    extra_args: Optional[str] = None,
    remove_origin_file: bool = True,
) -> bool:
    """
    Example:
    wav flac flac
    wav ogg ogg -ab 320k
    """
    # Spawn Tasks
    processes: List[Tuple[str, Optional[subprocess.Popen]]] = []
    for file_name in os.listdir(dir):
        file_path = f"{dir}/{file_name}"
        if not os.path.isfile(file_path):
            continue
        # Check ext
        if not file_name.lower().endswith("." + input_ext):
            continue
        cmd = _get_audio_precess_cmd(
            file_path,
            input_ext,
            output_ext,
            audio_format,
            extra_args,
        )
        if len(cmd) == 0:
            processes.append((file_path, None))
            continue
        process = subprocess.Popen(
            cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
        )
        processes.append((file_path, process))

    # Check Tasks
    has_error = False
    while len(processes) > 0:
        processes_waiting: List[Tuple[str, Optional[subprocess.Popen]]] = []
        for i, (file_path, process) in enumerate(processes):
            # Empty Process
            if process is None:
                if remove_origin_file and os.path.isfile(file_path):
                    os.remove(file_path)
                continue
            # Get status
            stdout, stderr = process.communicate()
            # Not Finished?
            poll_result = process.poll()
            if poll_result is None:
                processes_waiting.append((file_path, process))
                continue
            # Has Error?
            if poll_result != 0:
                has_error = True
                print(" - Has error!", file_path, stderr, "->", i)
            # Normal End: Print
            else:
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
