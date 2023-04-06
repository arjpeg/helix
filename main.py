import sys

from helix import Lexer, Parser

debug_attrs = {"print_tokens": 0}


def repl() -> None:
    try:
        while True:
            # Get user input
            text = input("helix > ")

            # Lex the input
            lexer = Lexer(text)
            tokens = lexer.generate_tokens()

            if debug_attrs.get("print_tokens"):
                print("Tokens in stream:")

                for token in tokens:
                    print("\t" + str(token))

            parser = Parser(list(tokens))
            ast = parser.parse()

            print(ast)

    except KeyboardInterrupt:
        pass


def run_file() -> None:
    file_name = sys.argv[1]

    with open(file_name, "r") as f:
        code = f.read()

    lexer = Lexer(code)
    tokens = lexer.generate_tokens()

    if debug_attrs.get("print_tokens"):
        print("Tokens in stream:")

        for token in tokens:
            print("\t" + str(token))

    parser = Parser(list(tokens))
    ast = parser.parse()

    print(ast)


if __name__ == "__main__":
    if len(sys.argv) > 1:
        run_file()

    else:
        repl()
