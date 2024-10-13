import webbrowser

tips = "Input id (default: Jump to List):"

print("Press Ctrl+C to Quit.")

while True:
    num_str = input(tips).strip()
    if len(num_str.split(" ")) == 2:
        nums = num_str.split()
        start, end = int(nums[0]), int(nums[1])
        if start > end:
            start, end = end, start
        for id in range(start, end + 1):
            webbrowser.open_new_tab(
                f"https://manbow.nothing.sh/event/event.cgi?action=More_def&num={id}&event=146"
            )

    elif len(num_str) > 0:
        if num_str.isdigit():
            print(f"Open no.{num_str}")
            webbrowser.open_new_tab(
                f"https://manbow.nothing.sh/event/event.cgi?action=More_def&num={num_str}&event=146"
            )
        else:
            print("Please input vaild number.")
    else:
        print("Open BMS List.")
        webbrowser.open_new_tab(
            "https://manbow.nothing.sh/event/event.cgi?action=sp&event=146"
        )
