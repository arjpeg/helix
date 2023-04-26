import sys

from helix import Interpreter, Lexer, Parser
from helix.helix_nodes import BlockNode

debug_attrs = {"print_tokens": 0}


def run(code: str):
    lexer = Lexer(code)
    tokens = lexer.generate_tokens()

    if debug_attrs.get("print_tokens"):
        print("Tokens in stream:")

        for token in tokens:
            print("\t" + str(token))

    parser = Parser(list(tokens))
    ast = parser.parse()

    if isinstance(ast, BlockNode) and len(ast.statements) == 1:
        ast = ast.statements[0]  # Unwrap the block node

    if isinstance(ast, BlockNode):
        Interpreter().visit(ast)
    else:
        if result := Interpreter().visit(ast):
            print(result)


def repl() -> None:
    try:
        while True:
            # Get user input
            text = input("helix > ")

            run(text)

    except KeyboardInterrupt:
        pass


def run_file() -> None:
    file_name = sys.argv[1]

    with open(file_name, "r") as f:
        code = f.read()

    run(code)


if __name__ == "__main__":
    if len(sys.argv) > 1:
        run_file()

    else:
        repl()
