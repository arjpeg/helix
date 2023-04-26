from typing import Any

from helix.data.object import Object
from helix.data.string import String


class Dict(Object):
    def __init__(self, elements: dict[str, Any]):
        self.elements = elements

    def add(self, other: Any):
        if isinstance(other, Dict):
            return Dict({**self.elements, **other.elements})

    def index(self, index: Any):
        if index in self.elements:
            return self.elements[index]

        if isinstance(index, String):
            if index.value in self.elements:
                return self.elements[index.value]

        raise Exception(f"Key '{index}' not found")

    def set_property(self, name: Any, value: Any):
        self.elements[name] = value

    def get_property(self, name: Any):
        return self.elements[name]

    def contains(self, other: Any):
        return other in self.elements

    def __repr__(self):
        result = "{\n"

        for key, value in self.elements.items():
            value_str = str(value).replace("\n", "\n\t")

            result += f"\t{key}: {value_str}\n"

        result += "}"

        return result
