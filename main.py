import options


def main():
    selection = input("输入要启用的功能的下标：").strip()
    if not selection.isdigit():
        print("请重新输入")


if __name__ == "__main__":
    main()
