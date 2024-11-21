from bms_fs import get_bms_folder_dir
from bms_media.audio import (
    AUDIO_PRESET_OGG_Q10,
)
from bms_media.video import (
    VIDEO_PRESET_MPEG1VIDEO_512X512,
    VIDEO_PRESET_AVI_512X512,
    VIDEO_PRESET_WMV2_512X512,
)
import scripts_bms_folder.transfer_audio
import scripts_bms_folder.transfer_video


def main():
    print("This file is for parsing HQ version to LQ version. Just for LR2 players.")
    root_dir = get_bms_folder_dir()
    # Parse Audio
    print("Parsing Audio... Phase 1: FLAC -> OGG")
    scripts_bms_folder.transfer_audio.main(
        root_dir=root_dir,
        input_ext=["flac"],
        transfer_mode=[AUDIO_PRESET_OGG_Q10],
        remove_origin_file=True,
        skip_on_fail=False,
    )
    # Parse Audio
    print("Parsing Video...")
    scripts_bms_folder.transfer_video.main(
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
    main()
