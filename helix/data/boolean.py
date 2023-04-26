from helix.data.object import Object


class Boolean(Object):
    def __init__(self, value: bool):
        self.value = value

    def __repr__(self):
        return str(self.value).lower()
