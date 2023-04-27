from helix.data.function import BuiltInFunction
from helix.data.object import Object
from helix.data.string import String


class File(Object):
    def __init__(self, path: String):
        self.path = path
        self.fd = None

        # methods
        self.set_property("open", BuiltInFunction("open", self.open))
        self.set_property("close", BuiltInFunction("close", self.close))

        self.set_property("read", BuiltInFunction("read", self.read))
        self.set_property("write", BuiltInFunction("write", self.write))

    def open(self, mode: String):
        self.fd = open(self.path.value, mode.value)

    def read(self):
        if not self.fd:
            raise Exception("File not open")

        return String(self.fd.read())

    def write(self, content: String):
        if not self.fd:
            raise Exception("File not open")

        # make sure we are in write mode
        if "w" not in self.fd.mode:
            raise Exception("Cannot write to file in read mode")

        self.fd.write(content.value)

    def close(self):
        assert self.fd

        self.fd.close()

    def __repr__(self) -> str:
        return f"File({self.path})"


file_namespace = BuiltInFunction("file", File)
