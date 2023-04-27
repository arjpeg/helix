from helix.data.null import Null
from helix.helix_symbol_table import SymbolTable

from .io_namespace import io_namespace
from .math_namespace import math_namespace

GLOBAL_SYMBOL_TABLE = SymbolTable(
    {
        "null": Null(),
        "io": io_namespace,
        "Math": math_namespace,
    }
)
