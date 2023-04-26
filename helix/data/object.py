from typing import Any


class Object:
    def set_property(self, name: Any, value: Any):
        setattr(self, name, value)

    def get_property(self, name: Any):
        return getattr(self, name)
