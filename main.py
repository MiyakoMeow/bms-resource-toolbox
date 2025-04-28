import options.bms_events
import options.bms_folder
import options.bms_folder_bigpack
import options.bms_folder_event
import options.rawpack
import scripts.pack

OPTIONS = (
    options.bms_events.OPTIONS
    + options.bms_folder.OPTIONS
    + options.bms_folder_bigpack.OPTIONS
    + options.bms_folder_event.OPTIONS
    + options.rawpack.OPTIONS
    + scripts.pack.OPTIONS
)


def main():
    print("功能列表如下：")
    for i, option in enumerate(OPTIONS):
        print(f" - {i}: {option.name if option.name else option.func.__name__}")
    selection = input("输入要启用的功能的下标：").strip()
    while not selection.isdigit():
        print("请重新输入")
        selection = input("输入要启用的功能的下标：").strip()

    selection = int(selection)
    OPTIONS[selection].exec()


if __name__ == "__main__":
    main()
