from typing import Any


class Variable:
    def __init__(self, value: Any, is_const: bool = False) -> None:
        self.value = value
        self.is_const = is_const


class SymbolTable:
    def __init__(self, symbols: dict[str, Any] | None = None) -> None:
        # the last element in the list is the current scope
        # each previous element is a parent scope
        self.symbols: list[dict[str, Variable]] = [{}]

        if symbols is not None:
            for name, value in symbols.items():
                self.symbols[0][name] = Variable(value)

    def get(self, name: str) -> Any:
        # loop through all scopes (in reverse order)
        for scope in reversed(self.symbols):
            if name in scope:
                return scope[name].value

        return None

    def set(self, name: str, value: Any, is_const: bool = False) -> None:
        self.symbols[-1][name] = Variable(value, is_const)

    def update(self, name: str, value: Any):
        for scope in reversed(self.symbols):
            if name in scope:
                if scope[name].is_const:
                    raise Exception(f"Cannot update const variable {name}")

                scope[name].value = value

                return

    def remove(self, name: str) -> None:
        if name in self.symbols[-1]:
            del self.symbols[-1][name]

    def push_scope(self) -> None:
        self.symbols.append({})

    def pop_scope(self) -> None:
        self.symbols.pop()
