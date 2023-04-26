from typing import Any

from helix.data.boolean import Boolean
from helix.data.function import BuiltInFunction
from helix.data.number import Number
from helix.data.object import Object


class String(Object):
    def __init__(self, value: str):
        self.value = value

        # methods
        self.set_property("length", Number(len(value)))
        self.set_property("to_int", BuiltInFunction("to_int", self.to_int))

    def add(self, other: Any):
        if isinstance(other, String):
            return String(self.value + other.value)

    def mul(self, other: Any):
        if isinstance(other, Number):
            if other.value < 0:
                raise Exception("Cannot multiply string by negative number")

            if other.value == 0:
                return String("")

            if isinstance(other.value, float):
                raise Exception("Cannot multiply string by float")

            return String(self.value * other.value)

    def equals(self, other: Any):
        if isinstance(other, String):
            return Boolean(self.value == other.value)

        return Boolean(False)

    def contains(self, other: Any):
        if isinstance(other, String):
            return Boolean(other.value in self.value)

        return Boolean(False)

    def to_int(self):
        return Number(int(self.value))

    def __repr__(self):
        return f'"{self.value}"'
