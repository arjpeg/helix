from typing import TypedDict

from helix.data.null import Null
from helix.data.object import Object
from helix.helix_symbol_table import SymbolTable


class FunctionContext(TypedDict):
    fn_name: str
    return_value: Object
    should_return: bool


class Context:
    def __init__(self, symbol_table: SymbolTable) -> None:
        self.symbol_table = symbol_table
        self.fn_context_stack: list[FunctionContext] = []

        self.in_var_declaration = False

    def push_fn_context(self, fn_name: str) -> None:
        self.fn_context_stack.append(
            {"fn_name": fn_name, "return_value": Null(), "should_return": False}
        )

    def pop_fn_context(self) -> FunctionContext:
        return self.fn_context_stack.pop()

    def get_fn_context(self) -> FunctionContext:
        return self.fn_context_stack[-1]
