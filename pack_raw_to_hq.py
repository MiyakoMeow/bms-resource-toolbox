from bms_media.audio import (
    AUDIO_PRESET_FLAC,
    AUDIO_PRESET_FLAC_FFMPEG,
)
from scripts_bms_folder import transfer_audio
from scripts_bms_folder import remove_unneed_media_file


def main():
    print(
        "This file is for parsing Raw version to HQ version. Just for beatoraja/Qwilight players."
    )
    root_dir = input("Input BMS Dir:")
    # Parse Audio
    print("Parsing Audio... Phase 1: WAV -> FLAC")
    transfer_audio.main(
        root_dir=root_dir,
        input_ext=["wav"],
        transfer_mode=[AUDIO_PRESET_FLAC, AUDIO_PRESET_FLAC_FFMPEG],
        remove_origin_file=True,
        skip_on_fail=False,
    )
    # Remove Unneed Media File
    print("Removing Unneed Files")
    remove_unneed_media_file.main(
        root_dir, preset=remove_unneed_media_file.PRESET_NORMAL
    )


if __name__ == "__main__":
    main()
