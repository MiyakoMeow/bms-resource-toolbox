import os

import openpyxl

from bms import get_dir_bms_info
from bms_fs import get_bms_folder_dir

if __name__ == "__main__":
    print("Set default dir by env BOFTT_DIR")

    root_dir = get_bms_folder_dir()

    # 创建一个 workbook
    workbook = openpyxl.Workbook()
    workbook.create_sheet("BMS List")

    worksheet = workbook["BMS List"]

    # 访问目录下的BMS文件夹
    for dir_name in os.listdir(root_dir):
        dir_path = os.path.join(root_dir, dir_name)
        if not os.path.isdir(dir_path):
            continue
        # 获得BMS信息
        info = get_dir_bms_info(dir_path)
        if info is None:
            continue
        # 获得目录编号
        id = dir_name.split(".")[0]
        # 填充信息
        worksheet[f"A{id}"] = id
        worksheet[f"B{id}"] = info.title
        worksheet[f"C{id}"] = info.artist
        worksheet[f"D{id}"] = info.genre

    # 保存 Excel 文件
    table_path = os.path.join(root_dir, "bms_list.xlsx")
    print(f"Saving table to {table_path}")
    workbook.save(table_path)
