# load all files in ./example and make sure they are valid

import os

from src import Lexer, Parser

files = []

files = [
    os.path.join("examples", f)
    for f in os.listdir("./examples")
    if f.endswith(".helix")
]


print(files)

# loop through all the files and see if they compile without errors
for file in files:
    print(f"Testing ./{file}...")

    with open(file, "r") as f:
        source = f.read()

    try:
        lexer = Lexer(source)
        tokens = lexer.generate_tokens()

        print("Lexing complete!")

        parser = Parser(list(tokens))
        ast = parser.parse()

        print("Success!\n")

    except Exception as e:
        print(f"Failed: {e}")
        exit(1)
