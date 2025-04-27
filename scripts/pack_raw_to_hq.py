from media.audio import (
    AUDIO_PRESET_FLAC,
    AUDIO_PRESET_FLAC_FFMPEG,
    bms_folder_transfer_audio,
)
from options.bms_folder_bigpack import (
    REMOVE_MEDIA_RULE_ORAJA,
    remove_unneed_media_files,
)


def activate():
    print(
        "This file is for parsing Raw version to HQ version. Just for beatoraja/Qwilight players."
    )
    root_dir = input("Input BMS Dir:")
    # Parse Audio
    print("Parsing Audio... Phase 1: WAV -> FLAC")
    bms_folder_transfer_audio(
        root_dir=root_dir,
        input_ext=["wav"],
        transfer_mode=[AUDIO_PRESET_FLAC, AUDIO_PRESET_FLAC_FFMPEG],
        remove_origin_file_when_success=True,
        remove_origin_file_when_failed=True,
        skip_on_fail=False,
    )
    # Remove Unneed Media File
    print("Removing Unneed Files")
    remove_unneed_media_files(root_dir, rules=REMOVE_MEDIA_RULE_ORAJA)


if __name__ == "__main__":
    activate()
