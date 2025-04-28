from enum import Enum, auto
from typing import Any, Callable, List, Tuple

from fs.input import input_path


class InputType(Enum):
    Any = auto()
    Word = auto()
    Int = auto()
    Path = auto()
    Confirm = auto()


def input_arg(type: InputType, tips: str) -> Any:
    match type:
        case InputType.Any:
            return input(tips)
        case InputType.Word:
            w_str = input(tips)
            while w_str.find(" ") != -1:
                print("Requires a word. Re-input.")
                w_str = input(tips)
            return w_str
        case InputType.Int:
            w_str = input(tips)
            while not w_str.isdigit():
                print("Requires a number. Re-input.")
                w_str = input(tips)
            return int(w_str)
        case InputType.Path:
            return input_path(tips)
        case InputType.Confirm:
            return input(f"{tips} [y/N]:").lower().startswith("y")


def exec_option(option: Tuple[Callable, List[Tuple[InputType, str]]]):
    inputs = []
    for input_type, tips in option[1]:
        inputs.append(input_arg(input_type, tips))
    option[0](*inputs)
