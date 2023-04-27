from typing import Any

from helix.data.boolean import Boolean
from helix.data.function import BuiltInFunction
from helix.data.number import Number
from helix.data.object import Object


class List(Object):
    def __init__(self, elements: list[Any]):
        self.elements = elements

        # methods
        self.set_property("length", BuiltInFunction("length", self.length))

    def add(self, other: Any):
        if isinstance(other, List):
            return List(self.elements + other.elements)

        if isinstance(other, Number):
            return List(self.elements + [other])

    def mul(self, other: Any):
        if isinstance(other, Number):
            if other.value < 0:
                raise Exception("Cannot multiply list by negative number")

            if other.value == 0:
                return List([])

            if isinstance(other.value, float):
                raise Exception("Cannot multiply list by float")

            return List(self.elements * other.value)

    def index(self, index: Any):
        if isinstance(index, Number):
            if isinstance(index.value, float):
                raise Exception("Cannot index list with float")

            return self.elements[index.value]

        raise Exception("Index must be a number")

    def contains(self, other: Any):
        for element in self.elements:
            if element.equals(other).value:
                return Boolean(True)

        return Boolean(False)

    def iter(self):
        iter_index = 0

        while iter_index < len(self.elements):
            yield self.elements[iter_index]

            iter_index += 1

    def length(self):
        return Number(len(self.elements))

    def __repr__(self):
        return f"[{', '.join(map(str, self.elements))})"
