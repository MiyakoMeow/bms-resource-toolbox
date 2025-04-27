#
#
# def _remove_unneed_media_files_entry(
#     root_dir: str, preset: List[Tuple[List[str], List[str]]] = []
# ):
#     # Select Preset
#     if len(preset) == 0:
#         for i, preset in enumerate(PRESETS):
#             print(f"- {i}: {PRESETS[i]}")
#         selection_str = input("Select Preset (Default: 0):")
#         selection = 0
#         if len(selection_str) > 0:
#             selection = int(selection_str)
#         preset = PRESETS[selection]
#     print(f"Selected: {preset}")
#
#     # Do
#     for bms_dir_name in os.listdir(root_dir):
#         bms_dir_path = os.path.join(root_dir, bms_dir_name)
#         if not os.path.isdir(bms_dir_path):
#             continue
#         remove_unneed_media_files(
#             bms_dir_path,
#             preset,
#         )
