from bms_media.audio import (
    AUDIO_PRESET_OGG_Q10,
    AUDIO_PRESET_WAV_FFMPEG,
    AUDIO_PRESET_WAV_FROM_FLAC,
)
from bms_media.video import (
    VIDEO_PRESET_MPEG1VIDEO_512X512,
    VIDEO_PRESET_WMV2_512X512,
)
import f_transfer_audio
import f_transfer_video


def main():
    print("This file is for parsing HQ version to LQ version. Just for LR2 players.")
    root_dir = input("Input BMS Dir:")
    # Parse Audio
    print("Parsing Audio... Phase 1: FLAC -> WAV")
    f_transfer_audio.main(
        root_dir=root_dir,
        input_ext=["flac"],
        transfer_mode=[AUDIO_PRESET_WAV_FROM_FLAC, AUDIO_PRESET_WAV_FFMPEG],
        remove_origin_file=True,
    )
    # Parse Audio
    print("Parsing Audio... Phase 2: WAV -> OGG")
    f_transfer_audio.main(
        root_dir=root_dir,
        input_ext=["wav"],
        transfer_mode=[AUDIO_PRESET_OGG_Q10],
        remove_origin_file=True,
    )
    # Parse Audio
    print("Parsing Video...")
    f_transfer_video.main(
        root_dir=root_dir,
        input_exts=["mp4", "avi"],
        presets=[VIDEO_PRESET_MPEG1VIDEO_512X512, VIDEO_PRESET_WMV2_512X512],
        remove_origin_file=True,
        use_prefered=False,
    )


if __name__ == "__main__":
    main()
