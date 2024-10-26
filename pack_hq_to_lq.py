from bms_media.audio import AUDIO_PRESET_OGG_Q10
from bms_media.video import (
    VIDEO_PRESET_MPEG1VIDEO_512X512,
    VIDEO_PRESET_WMV2_512X512,
)
import f_transfer_audio
import f_transfer_video


def main():
    print("This file is for parsing HQ version to LQ version. Just for LR2 players.")
    # Parse Audio
    print("Parsing Audio... Phase 1:")
    f_transfer_audio.main(["flac"], [AUDIO_PRESET_OGG_Q10], remove_origin_file=True)
    # Parse Audio
    print("Parsing Video...")
    f_transfer_video.main(
        ["mp4", "avi"],
        [VIDEO_PRESET_WMV2_512X512, VIDEO_PRESET_MPEG1VIDEO_512X512],
        remove_origin_file=True,
        use_prefered=False,
    )


if __name__ == "__main__":
    main()
