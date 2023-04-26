from typing import Any

from helix.data.object import Object


class Null(Object):
    def __repr__(self):
        return "Null"

    def set_property(self, name: Any, value: Any):
        raise Exception("Cannot set property on null")

    def get_property(self, name: Any):
        raise Exception("Cannot get property on null")
