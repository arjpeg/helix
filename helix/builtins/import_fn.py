from typing import Any, Callable

from helix.data.dict import Dict
from helix.data.function import PythonFunction
from helix.data.string import String
from helix.helix_context import Context
from helix.helix_nodes import ASTNode


def _import_fn(
    args: list[String], context: Context, visitor_method: Callable[[ASTNode], Any]
):
    from helix import run
    from helix.builtins.global_symbol_table import GLOBAL_SYMBOL_TABLE

    assert len(args) == 1, f"Expected 1 argument for import, but got {len(args)}"

    # get the file name
    file_name = args[0]

    # get the file contents
    with open(file_name.value, "r") as f:
        code = f.read()

    # create a new context
    new_context = Context(GLOBAL_SYMBOL_TABLE)

    # run the code (to get the context)
    _, new_context = run(code, new_context)

    # see if the function was called in a variable declaration
    if context.in_var_declaration:
        # return the variables in a Dict
        vars = new_context.symbol_table

        result = Dict(
            {var: vars.get(var) for var in vars.symbols[-1]}  # should only be one scope
        )

        return result

    # add all the variables to the current context
    for var in new_context.symbol_table.symbols[-1]:
        context.symbol_table.set(var, context.symbol_table.get(var))


import_fn = PythonFunction("import", _import_fn)
