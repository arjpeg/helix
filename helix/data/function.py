from typing import Any, Callable

from helix.data.object import Object
from helix.helix_context import Context
from helix.helix_nodes import ASTNode, BlockNode, ReturnNode


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
        context: Context,
        visitor_method: Callable[[ASTNode], Any],
    ):
        # call the function
        context.symbol_table.push_scope()

        if len(args) != len(self.args):
            raise Exception(
                f"Expected {len(self.args)} arguments for function {self.name}, but got {len(args)}"
            )

        # add all the arguments to the symbol table
        for i in range(len(args)):
            context.symbol_table.set(self.args[i], args[i])


        visitor_method(self.body)
        context.symbol_table.pop_scope()

        # at this point, the return value will be in the context
        result = context.current_scope().return_value

        return result

    def __repr__(self):
        return f"Function(<{self.name}>, {self.args})"


class BuiltInFunction(Object):
    def __init__(self, name: str, code: Any) -> None:
        self.name = name
        self.code = code

    def call(
        self,
        args: list[str],
        context: Context,
        visitor_method: Callable[[ASTNode], Any],
    ):
        res = self.code(*args)

        if not len(context.scopes):
            raise Exception(
                "WTFFFFF, this should not happen, this is a bug in the interpreter"
            )

        if res:
            context.current_scope().should_return = True
            context.current_scope().return_value = res

        return res


class PythonFunction(Object):
    def __init__(
        self,
        name: str,
        code: Any,
    ) -> None:
        """
        This is different from a built-in function because it is a function that is written in python, and needs the context to be passed in.
        """
        self.name = name
        self.fn = code

    def call(
        self,
        args: list[str],
        context: Context,
        visitor_method: Callable[[ASTNode], Any],
    ):
        return self.fn(args, context, visitor_method)
