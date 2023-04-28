from typing import Any

from helix.helix_context import Context
from helix.helix_nodes import BlockNode

from .helix_interpreter import Interpreter  # type: ignore
from .helix_lexer import Lexer  # type: ignore
from .helix_parser import Parser  # type: ignore


def run(code: str, context: Context | None = None) -> tuple[Any | None, Context]:
    lexer = Lexer(code)
    tokens = lexer.generate_tokens()

    parser = Parser(list(tokens))
    ast = parser.parse()

    if isinstance(ast, BlockNode) and len(ast.statements) == 1:
        ast = ast.statements[0]  # Unwrap the block node

    interpreter = Interpreter(context)

    if context is None:
        context = interpreter.context

    if isinstance(ast, BlockNode):
        interpreter.visit(ast)
        return (None, context)

    if result := interpreter.visit(ast):
        return (result, context)

    return (None, context)


__all__ = ["run"]
