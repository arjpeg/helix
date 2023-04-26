from typing import Any

from helix.data.boolean import Boolean
from helix.data.object import Object


class Number(Object):
    def __init__(self, value: int | float):
        self.value = value

    def add(self, other: Any):
        if isinstance(other, Number):
            return Number(self.value + other.value)

    def sub(self, other: Any):
        if isinstance(other, Number):
            return Number(self.value - other.value)

    def mul(self, other: Any):
        if isinstance(other, Number):
            return Number(self.value * other.value)

    def div(self, other: Any):
        if isinstance(other, Number):
            return Number(self.value / other.value)

    def pow(self, other: Any):
        if isinstance(other, Number):
            return Number(self.value**other.value)

    def equals(self, other: Any):
        if isinstance(other, Number):
            return Boolean(self.value == other.value)

        return Boolean(False)

    def greater_than(self, other: Any):
        if isinstance(other, Number):
            return Boolean(self.value > other.value)

        return Boolean(False)

    def less_than(self, other: Any):
        if isinstance(other, Number):
            return Boolean(self.value < other.value)

        return Boolean(False)

    def greater_than_equals(self, other: Any):
        if isinstance(other, Number):
            return Boolean(self.value >= other.value)

        return Boolean(False)

    def less_than_equals(self, other: Any):
        if isinstance(other, Number):
            return Boolean(self.value <= other.value)

        return Boolean(False)

    def __repr__(self):
        return f"{self.value}"
