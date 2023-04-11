# used in the interpreter


from typing import Any, Callable

from helix.helix_nodes import ASTNode, BlockNode, ReturnNode
from helix.helix_symbol_table import SymbolTable


class Object:
    def set_property(self, name: Any, value: Any):
        setattr(self, name, value)

    def get_property(self, name: Any):
        return getattr(self, name)


class Null(Object):
    def __repr__(self):
        return "Null"

    def set_property(self, name: Any, value: Any):
        raise Exception("Cannot set property on null")

    def get_property(self, name: Any):
        raise Exception("Cannot get property on null")


class Boolean(Object):
    def __init__(self, value: bool):
        self.value = value

    def __repr__(self):
        return str(self.value).lower()


class Number(Object):
    def __init__(self, value: int | float):
        self.value = value

    def add(self, other: Any):
        if isinstance(other, Number):
            return Number(self.value + other.value)

    def sub(self, other: Any):
        if isinstance(other, Number):
            return Number(self.value - other.value)

    def mul(self, other: Any):
        if isinstance(other, Number):
            return Number(self.value * other.value)

    def div(self, other: Any):
        if isinstance(other, Number):
            return Number(self.value / other.value)

    def pow(self, other: Any):
        if isinstance(other, Number):
            return Number(self.value**other.value)

    def equals(self, other: Any):
        if isinstance(other, Number):
            return Boolean(self.value == other.value)

        return Boolean(False)

    def greater_than(self, other: Any):
        if isinstance(other, Number):
            return Boolean(self.value > other.value)

        return Boolean(False)

    def less_than(self, other: Any):
        if isinstance(other, Number):
            return Boolean(self.value < other.value)

        return Boolean(False)

    def greater_than_equals(self, other: Any):
        if isinstance(other, Number):
            return Boolean(self.value >= other.value)

        return Boolean(False)

    def less_than_equals(self, other: Any):
        if isinstance(other, Number):
            return Boolean(self.value <= other.value)

        return Boolean(False)

    def __repr__(self):
        return f"{self.value}"


class String(Object):
    def __init__(self, value: str):
        self.value = value

        # methods
        self.set_property("length", Number(len(value)))
        self.set_property("to_int", BuiltInFunction("to_int", self.to_int))

    def add(self, other: Any):
        if isinstance(other, String):
            return String(self.value + other.value)

    def mul(self, other: Any):
        if isinstance(other, Number):
            if other.value < 0:
                raise Exception("Cannot multiply string by negative number")

            if other.value == 0:
                return String("")

            if isinstance(other.value, float):
                raise Exception("Cannot multiply string by float")

            return String(self.value * other.value)

    def equals(self, other: Any):
        if isinstance(other, String):
            return Boolean(self.value == other.value)

        return Boolean(False)

    def contains(self, other: Any):
        if isinstance(other, String):
            return Boolean(other.value in self.value)

        return Boolean(False)

    def to_int(self):
        return Number(int(self.value))

    def __repr__(self):
        return f'"{self.value}"'


class List(Object):
    def __init__(self, elements: list[Any]):
        self.elements = elements

    def add(self, other: Any):
        if isinstance(other, List):
            return List(self.elements + other.elements)

        if isinstance(other, Number):
            return List(self.elements + [other])

    def mul(self, other: Any):
        if isinstance(other, Number):
            if other.value < 0:
                raise Exception("Cannot multiply list by negative number")

            if other.value == 0:
                return List([])

            if isinstance(other.value, float):
                raise Exception("Cannot multiply list by float")

            return List(self.elements * other.value)

    def index(self, index: Any):
        if isinstance(index, Number):
            if isinstance(index.value, float):
                raise Exception("Cannot index list with float")

            return self.elements[index.value]

        raise Exception("Index must be a number")

    def contains(self, other: Any):
        for element in self.elements:
            if element.equals(other).value:
                return Boolean(True)

        return Boolean(False)

    def iter(self):
        iter_index = 0

        while iter_index < len(self.elements):
            yield self.elements[iter_index]

            iter_index += 1

    def __repr__(self):
        return f"List([{', '.join(map(str, self.elements))}])"


class Dict(Object):
    def __init__(self, elements: dict[str, Any]):
        self.elements = elements

    def add(self, other: Any):
        if isinstance(other, Dict):
            return Dict({**self.elements, **other.elements})

    def index(self, index: Any):
        if index in self.elements:
            return self.elements[index]

        if isinstance(index, String):
            if index.value in self.elements:
                return self.elements[index.value]

        raise Exception(f"Key '{index}' not found")

    def set_property(self, name: Any, value: Any):
        self.elements[name] = value

    def get_property(self, name: Any):
        return self.elements[name]

    def contains(self, other: Any):
        return other in self.elements

    def __repr__(self):
        result = "{\n"

        for key, value in self.elements.items():
            value_str = str(value).replace("\n", "\n\t")

            result += f"\t{key}: {value_str}\n"

        result += "}"

        return result


class Tuple(Object):
    def __init__(self, elements: list[Any]):
        self.elements = elements

    def contains(self, other: Any):
        return other in self.elements

    def __repr__(self):
        return f"Tuple({', '.join(map(str, self.elements))})"


class Function(Object):
    def __init__(
        self,
        name: str,
        args: list[str],
        body: BlockNode | ReturnNode,
    ) -> None:
        self.name = name
        self.args = args
        self.body = body

    def call(
        self,
        args: list[str],
        symbol_table: SymbolTable,
        visitor_method: Callable[[ASTNode], Any],
    ):
        print(f"in function {self.name} call, being passed {args}")

        # call the function
        symbol_table.push_scope()

        if len(args) != len(self.args):
            raise Exception(
                f"Expected {len(self.args)} arguments for function {self.name}, but got {len(args)}"
            )

        symbol_table.push_scope()

        # add all the arguments to the symbol table
        for i in range(len(args)):
            symbol_table.set(self.args[i], args[i])

        # execute the function
        # because there might be a return statement, we need to
        # manually execute the body
        if isinstance(self.body, ReturnNode):
            result = visitor_method(self.body)

            symbol_table.pop_scope()

            return result

        result = Null()

        for statement in self.body.statements:
            print("executing statement", statement)
            result = visitor_method(statement)

            if isinstance(statement, ReturnNode):
                break

        symbol_table.pop_scope()

        return result

    def __repr__(self):
        return f"Function({self.name}, {self.args}, {self.body})"


class BuiltInFunction(Object):
    def __init__(self, name: str, code: Any) -> None:
        self.name = name
        self.code = code

    def call(
        self,
        args: list[Any],
        symbol_table: SymbolTable,
        visitor_method: Callable[[ASTNode], Any],
    ):
        return self.code(*args)
