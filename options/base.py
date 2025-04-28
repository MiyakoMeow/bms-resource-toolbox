from enum import Enum, auto
from typing import Any, Callable, List, Tuple

from fs.input import input_path


class InputType(Enum):
    Any = auto()
    Word = auto()
    Int = auto()
    Path = auto()
    Confirm = auto()


def input_arg(type: InputType) -> Any:
    match type:
        case InputType.Any:
            return input("Input:")
        case InputType.Word:
            w_str = input("Input:")
            while w_str.find(" ") != -1:
                print("Requires a word. Re-input.")
                w_str = input("Input:")
            return w_str
        case InputType.Int:
            w_str = input("Input:")
            while not w_str.isdigit():
                print("Requires a number. Re-input.")
                w_str = input("Input:")
            return int(w_str)
        case InputType.Path:
            return input_path("Input:")
        case InputType.Confirm:
            return input(f"{'Input:'} [y/N]:").lower().startswith("y")


def exec_option(option: Tuple[Callable, List[Tuple[InputType, str]]]):
    args = []
    func, arg_defines = option
    for i, (input_type, tips) in enumerate(arg_defines):
        print(f"Input {i + 1}/{len(arg_defines)}, Type: {input_type}, Tips: {tips}")
        args.append(input_arg(input_type))
    func(*args)
