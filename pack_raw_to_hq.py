from bms_media.audio import (
    AUDIO_PRESET_FLAC,
    AUDIO_PRESET_FLAC_FFMPEG,
)
import f_transfer_audio
import f_remove_unneed_media_file


def main():
    print(
        "This file is for parsing Raw version to HQ version. Just for beatoraja/Qwilight players."
    )
    root_dir = input("Input BMS Dir:")
    # Parse Audio
    print("Parsing Audio... Phase 1: WAV -> FLAC")
    f_transfer_audio.main(
        root_dir=root_dir,
        input_ext=["wav"],
        transfer_mode=[AUDIO_PRESET_FLAC, AUDIO_PRESET_FLAC_FFMPEG],
        remove_origin_file=True,
        skip_on_fail=True,
    )
    # Remove Unneed Media File
    print("Removing Unneed Files")
    f_remove_unneed_media_file.main(
        root_dir, preset=f_remove_unneed_media_file.PRESET_NORMAL
    )


if __name__ == "__main__":
    main()
