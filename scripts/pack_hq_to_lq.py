from media.audio import (
    AUDIO_PRESET_OGG_Q10,
    bms_folder_transfer_audio,
)
from media.video import (
    VIDEO_PRESET_MPEG1VIDEO_512X512,
    VIDEO_PRESET_AVI_512X512,
    VIDEO_PRESET_WMV2_512X512,
    bms_folder_transfer_video,
)


def activate():
    print("This file is for parsing HQ version to LQ version. Just for LR2 players.")
    root_dir = input("Input BMS Dir:")
    # Parse Audio
    print("Parsing Audio... Phase 1: FLAC -> OGG")
    bms_folder_transfer_audio(
        root_dir=root_dir,
        input_ext=["flac"],
        transfer_mode=[AUDIO_PRESET_OGG_Q10],
        remove_origin_file_when_success=True,
        remove_origin_file_when_failed=False,
        skip_on_fail=False,
    )
    # Parse Audio
    print("Parsing Video...")
    bms_folder_transfer_video(
        root_dir=root_dir,
        input_exts=["mp4"],
        presets=[
            VIDEO_PRESET_MPEG1VIDEO_512X512,
            VIDEO_PRESET_WMV2_512X512,
            VIDEO_PRESET_AVI_512X512,
        ],
        remove_origin_file=True,
        use_prefered=False,
    )


if __name__ == "__main__":
    activate()
