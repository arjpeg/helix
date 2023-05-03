from helix.data.null import Null
from helix.helix_symbol_table import SymbolTable


class Scope:
    def __init__(self, symbol_table: SymbolTable, scope_name: str) -> None:
        """
        This class is used to keep track of the current function scope.
        It also keeps track of the current symbol table, in addition to whether
        we should return from the current function, should continue or break
        from the current loop, and whether we are currently in a variable declaration.
        """
        self.symbol_table = symbol_table

        self.should_continue = False
        self.should_break = False

        self.should_return = False
        self.return_value = Null()

        self.scope_name = scope_name

        self.in_var_declaration = False

    @property
    def should_stop(self) -> bool:
        return self.should_return or self.should_break or self.should_continue

    def __repr__(self) -> str:
        return f"Scope({self.scope_name})"


class Context:
    def __init__(self, symbol_table: SymbolTable) -> None:
        self.symbol_table = symbol_table
        self.scopes: list[Scope] = [Scope(symbol_table, "global")]

    def push_scope(self, fn_name: str) -> None:
        self.scopes.append(Scope(self.symbol_table, fn_name))
        self.scopes[-1].symbol_table.push_scope()

    def pop_scope(self) -> Scope:
        res = self.scopes.pop()
        res.symbol_table.pop_scope()

        return res

    def current_scope(self) -> Scope:
        """
        Only use this method if you are sure that there is a current scope.
        If there is no current scope, then this method will raise an IndexError.
        """
        return self.scopes[-1]
