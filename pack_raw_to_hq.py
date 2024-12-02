from bms_media.audio import (
    AUDIO_PRESET_FLAC,
    AUDIO_PRESET_FLAC_FFMPEG,
)
import bms_folder_transfer_audio
import bms_folder_remove_unneed_media_file


def main():
    print(
        "This file is for parsing Raw version to HQ version. Just for beatoraja/Qwilight players."
    )
    root_dir = input("Input BMS Dir:")
    # Parse Audio
    print("Parsing Audio... Phase 1: WAV -> FLAC")
    bms_folder_transfer_audio.main(
        root_dir=root_dir,
        input_ext=["wav"],
        transfer_mode=[AUDIO_PRESET_FLAC, AUDIO_PRESET_FLAC_FFMPEG],
        remove_origin_file_when_success=True,
        remove_origin_file_when_failed=True,
        skip_on_fail=False,
    )
    # Remove Unneed Media File
    print("Removing Unneed Files")
    bms_folder_remove_unneed_media_file.main(
        root_dir, preset=bms_folder_remove_unneed_media_file.PRESET_NORMAL
    )


if __name__ == "__main__":
    main()
