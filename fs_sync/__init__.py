import os
import shutil
from typing import List

from fs_sync.preset import SoftSyncPreset, SoftSyncExec, get_file_sha512


def sync_folder(
    src_dir: str,
    dst_dir: str,
    preset: SoftSyncPreset,
):
    src_list = os.listdir(src_dir)
    dst_list = os.listdir(dst_dir)

    # Cache For Print
    _src_copy_files: List[str] = []
    _src_move_files: List[str] = []
    _src_remove_files: List[str] = []
    _dst_remove_files: List[str] = []
    _dst_remove_dirs: List[str] = []

    # Src: Copy or Move or Remove
    for src_element in src_list:
        src_path = os.path.join(src_dir, src_element)
        dst_path = os.path.join(dst_dir, src_element)
        if os.path.isdir(src_path):
            # Src: Dir
            if os.path.isdir(dst_path):
                sync_folder(
                    src_path,
                    dst_path,
                    preset,
                )
            else:
                os.mkdir(dst_path)
                sync_folder(
                    src_path,
                    dst_path,
                    preset,
                )
        elif os.path.isfile(src_path):
            # Src: File
            # Check Ext
            ext_check_passed = preset.allow_other_exts
            ext = src_element.rsplit(".")[-1]
            if ext in preset.allow_src_exts:
                ext_check_passed = True
            if ext in preset.disallow_src_exts:
                ext_check_passed = False
            if not ext_check_passed:
                continue
            # Check Ext Bound
            ext_in_bound = False
            for (
                ext_bound_from_list,
                ext_bound_to_list,
            ) in preset.no_activate_ext_bound_pairs:
                if ext not in ext_bound_from_list:
                    continue
                # Found: Bound From
                for ext_bound_to in ext_bound_to_list:
                    bound_file_path = dst_path[: -len(ext)] + ext_bound_to
                    if not os.path.isfile(bound_file_path):
                        continue
                    # Found: Bound To
                    ext_in_bound = True
                    break
                if ext_in_bound:
                    break
            if ext_in_bound:
                continue
            # Replace: Check
            dst_file_exists = os.path.isfile(dst_path)
            is_same_file = dst_file_exists
            if preset.check_file_size and is_same_file and dst_file_exists:
                src_size = os.path.getsize(src_path)
                dst_size = os.path.getsize(dst_path)
                is_same_file = is_same_file and src_size == dst_size
            if preset.check_file_mtime and is_same_file and dst_file_exists:
                src_mtime = os.path.getmtime(src_path)
                dst_mtime = os.path.getmtime(dst_path)
                is_same_file = is_same_file and src_mtime == dst_mtime
            if preset.check_file_sha512 and is_same_file and dst_file_exists:
                src_value = get_file_sha512(src_path)
                dst_value = get_file_sha512(dst_path)
                is_same_file = is_same_file and src_value == dst_value
            # Replace: Exec
            if not dst_file_exists or not is_same_file:
                src_mtime = os.path.getmtime(src_path)
                match preset.exec:
                    case SoftSyncExec.NONE:
                        pass
                    case SoftSyncExec.COPY:
                        _src_copy_files.append(src_element)
                        shutil.copy(src_path, dst_path)
                        # Set atime/mtime
                        os.utime(dst_path, (src_mtime, src_mtime))
                    case SoftSyncExec.MOVE:
                        _src_move_files.append(src_element)
                        shutil.move(src_path, dst_path)
                        # Set atime/mtime
                        os.utime(dst_path, (src_mtime, src_mtime))
            # Remove same ori files
            if (
                preset.remove_src_same_files
                and dst_file_exists
                and is_same_file
                and os.path.isfile(src_path)
            ):
                _src_remove_files.append(src_element)
                os.remove(src_path)

    # Dst: Remove
    for dst_element in dst_list:
        if not preset.remove_dst_extra_files:
            break
        src_path = os.path.join(src_dir, dst_element)
        dst_path = os.path.join(dst_dir, dst_element)
        if os.path.isdir(dst_path):
            if os.path.isdir(src_path):
                pass
            else:
                _dst_remove_dirs.append(dst_element)
                shutil.rmtree(dst_path)
        elif os.path.isfile(dst_path):
            if not os.path.isfile(src_path):
                _dst_remove_files.append(dst_element)
                os.remove(dst_path)

    # Print
    if (
        len(_src_copy_files) > 0
        or len(_src_move_files) > 0
        or len(_src_remove_files) > 0
        or len(_dst_remove_files) > 0
        or len(_dst_remove_dirs) > 0
    ):
        print(f"{src_dir} -> {dst_dir}:")
        if len(_src_copy_files) > 0:
            print(f"Src copy: {_src_copy_files}")
        if len(_src_move_files) > 0:
            print(f"Src move: {_src_move_files}")
        if len(_src_remove_files) > 0:
            print(f"Src remove: {_src_remove_files}")
        if len(_dst_remove_files) > 0:
            print(f"Dst remove: {_dst_remove_files}")
        if len(_dst_remove_dirs) > 0:
            print(f"Dst remove dir: {_dst_remove_dirs}")
