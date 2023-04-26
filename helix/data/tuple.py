from typing import Any

from helix.data.object import Object


class Tuple(Object):
    def __init__(self, elements: list[Any]):
        self.elements = elements

    def contains(self, other: Any):
        return other in self.elements

    def __repr__(self):
        return f"Tuple({', '.join(map(str, self.elements))})"
