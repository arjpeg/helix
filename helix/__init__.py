from helix.helix_context import Context
from helix.helix_nodes import BlockNode

from .helix_interpreter import Interpreter  # type: ignore
from .helix_lexer import Lexer  # type: ignore
from .helix_parser import Parser  # type: ignore


def run(code: str, context: Context | None = None):
    lexer = Lexer(code)
    tokens = lexer.generate_tokens()

    parser = Parser(list(tokens))
    ast = parser.parse()

    if isinstance(ast, BlockNode) and len(ast.statements) == 1:
        ast = ast.statements[0]  # Unwrap the block node

    if isinstance(ast, BlockNode):
        Interpreter(context).visit(ast)
    else:
        if result := Interpreter(context).visit(ast):
            print(result)


__all__ = ["run"]
