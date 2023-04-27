import math

from helix.data.dict import Dict
from helix.data.function import BuiltInFunction
from helix.data.number import Number

math_namespace = Dict(
    {
        "pi": Number(math.pi),
        "e": Number(math.e),
        "sin": BuiltInFunction("sin", math.sin),
        "cos": BuiltInFunction("cos", math.cos),
        "tan": BuiltInFunction("tan", math.tan),
        "sqrt": BuiltInFunction("sqrt", lambda x: Number(math.sqrt(x.value))),  # type: ignore
    }
)
