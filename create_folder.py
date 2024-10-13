import os
import os.path

FOLDER_COUNT = 300

BOFTT_DIR = os.environ.get("BOFTT_DIR")
if BOFTT_DIR is None:
    BOFTT_DIR = os.path.abspath(".")

if __name__ == "__main__":
    print("Set default dir by env BOFTT_DIR")
    root_dir = input(f"Input root dir of bms dirs (Default: {BOFTT_DIR}):")
    if len(root_dir.strip()) == 0:
        root_dir = BOFTT_DIR

    folder_count = input(f"Input folder count (Default: {FOLDER_COUNT}):").strip()
    if len(folder_count) == 0 or not folder_count.isdigit():
        folder_count = FOLDER_COUNT
    folder_count = int(folder_count)

    existing_elements = os.listdir(root_dir)
    for element_name in existing_elements:
        path = f"{root_dir}/{element_name}"
        if not os.path.isdir(path):
            existing_elements.remove(element_name)

    for id in range(1, folder_count + 1):
        new_dir_name = f"{id}"
        id_exists = False
        for element_name in existing_elements:
            if element_name.startswith(f"{new_dir_name}"):
                id_exists = True
                break

        if id_exists:
            continue

        new_dir_path = f"{root_dir}/{new_dir_name}"
        if not os.path.isdir(new_dir_path):
            os.mkdir(new_dir_path)
