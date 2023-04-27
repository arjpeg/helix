from typing import Any

from helix.data.dict import Dict
from helix.data.function import BuiltInFunction
from helix.data.string import String


def custom_print(*args: Any):
    for arg in args:
        if isinstance(arg, String):
            print(arg.value, end=" ")
        else:
            print(arg, end=" ")

    print()


def custom_input(prompt: Any | None = None):
    if prompt:
        if isinstance(prompt, String):
            print(prompt.value, end="")
        else:
            print(prompt, end="")

    return String(input())


io_namespace = Dict(
    {
        "print": BuiltInFunction("print", custom_print),
        "input": BuiltInFunction("input", custom_input),
    }
)

__all__ = ["io_namespace"]
