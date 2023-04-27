import sys

from helix import run
from helix.builtins.global_symbol_table import GLOBAL_SYMBOL_TABLE
from helix.helix_context import Context


def repl() -> None:
    # create a new context - this will be used for allowing
    # the user to define variables and functions
    context = Context(GLOBAL_SYMBOL_TABLE)

    context.symbol_table.push_scope()

    try:
        while True:
            # Get user input
            text = input("helix > ")

            run(text, context)

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
