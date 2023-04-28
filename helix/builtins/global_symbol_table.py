from helix.builtins.import_fn import import_fn
from helix.data.function import BuiltInFunction
from helix.data.list import List
from helix.data.null import Null
from helix.data.number import Number
from helix.helix_symbol_table import SymbolTable

from .file_namespace import file_namespace
from .io_namespace import io_namespace
from .math_namespace import math_namespace


def range_fn(start: Number, end: Number | None = None):
    if end is None:
        end = start
        start = Number(0)

    if isinstance(start.value, float) or isinstance(end.value, float):
        raise Exception("range() expects integer arguments")

    return List([Number(i) for i in range(start.value, end.value)])


GLOBAL_SYMBOL_TABLE = SymbolTable(
    {
        "null": Null(),
        "io": io_namespace,
        "file": file_namespace,
        "Math": math_namespace,
        # methods
        "import": import_fn,
        "range": BuiltInFunction("range", range_fn),
    }
)
