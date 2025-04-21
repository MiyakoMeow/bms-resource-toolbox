from typing import Callable, List, Optional, Tuple


BIGPACK_FUNCTIONS: List[Tuple[str, Optional[Callable[[], None]]]] = [("", None)]
BMS_FOLDER_FUNCTIONS: List[Tuple[str, Optional[Callable[[], None]]]] = [("", None)]
RAWPACK_FUNCTIONS: List[Tuple[str, Optional[Callable[[], None]]]] = [("", None)]
UTIL_FUNCTIONS: List[Tuple[str, Optional[Callable[[], None]]]] = [("", None)]
FUNCTIONS: List[Tuple[str, Optional[Callable[[], None]]]] = (
    BIGPACK_FUNCTIONS + BMS_FOLDER_FUNCTIONS + BIGPACK_FUNCTIONS + UTIL_FUNCTIONS
)


def main():
    # TODO: finish func
    print("函数如下：")
    for index, (title, func) in enumerate(FUNCTIONS):
        print(f"{index} - {title}", "" if func else "空函数")

    print()
    selection = input("输入要启用的功能的下标：").strip()
    if not selection.isdigit():
        print("请重新输入")


if __name__ == "__main__":
    main()
