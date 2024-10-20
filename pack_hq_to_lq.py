from bms_media import VIDEO_PRESET_WMV_512X512
import f_transfer_audio
import f_transfer_video


def main():
    print("This file is for parsing HQ version to LQ version. Just for LR2 players.")
    # Parse Audio
    print("Parsing Audio... Phase 1:")
    audio_tran_mode = f_transfer_audio.MODES[2]
    if not audio_tran_mode[0].startswith("Reverse:"):
        print("LQ Mode Not Found!")
        return
    f_transfer_audio.main(
        audio_tran_mode[1], audio_tran_mode[2], remove_origin_file=True
    )
    print("Parsing Audio... Phase 1:")
    audio_tran_mode = f_transfer_audio.MODES[1]
    if not audio_tran_mode[0].startswith("Compress:"):
        print("LQ Mode Not Found!")
        return
    f_transfer_audio.main(
        audio_tran_mode[1], audio_tran_mode[2], remove_origin_file=True
    )
    # Parse Audio
    print("Parsing Video...")
    video_preset = VIDEO_PRESET_WMV_512X512
    f_transfer_video.main(video_preset, remove_origin_file=True)


if __name__ == "__main__":
    main()
