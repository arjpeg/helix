from helix.data.null import Null
from helix.helix_symbol_table import SymbolTable


class Context:
    def __init__(self, symbol_table: SymbolTable) -> None:
        self.symbol_table = symbol_table

        self.should_return = False
        self.return_value = Null()
