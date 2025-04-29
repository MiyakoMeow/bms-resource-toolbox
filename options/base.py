from enum import Enum, auto
from typing import Any, Callable, List
from dataclasses import dataclass, field

from fs.input import input_path


class InputType(Enum):
    Any = auto()
    Word = auto()
    Int = auto()
    Path = auto()
    Confirm = auto()


@dataclass
class Input:
    type: InputType = InputType.Any
    description: str = ""

    def input(self) -> Any:
        match self:
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


@dataclass
class Option:
    name: str
    func: Callable
    inputs: List[Input] = field(default_factory=list)

    def exec(self):
        print(self.name if self.name else self.func.__name__)
        args = []
        for i, input in enumerate(self.inputs):
            print(
                f"Input {i + 1}/{len(self.inputs)}, Type: {input.type}, Desc: {input.description}"
            )
            args.append(input.input())
        self.func(*args)
