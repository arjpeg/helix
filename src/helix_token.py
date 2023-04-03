from enum import Enum, auto
from typing import Generic, TypeVar


class Keyword(Enum):
    """
    Enum to represent the keywords.
    """

    LET = "let"
    IF = "if"
    ELSE = "else"

    FOR = "for"
    WHILE = "while"

    IN = "in"

    FUNCTION = "fn"


class TokenType(Enum):
    """
    Enum to represent the types of tokens.
    """

    # Data types
    INT = auto()
    FLOAT = auto()
    STRING = auto()

    IDENTIFIER = auto()
    KEYWORD = auto()
    PUNCTUATION = auto()

    NEWLINE = auto()
    EOF = auto()

    # Operators
    PLUS = "+"
    MINUS = "-"
    MUL = "*"
    DIV = "/"

    LPAREN = "("
    RPAREN = ")"

    # Assignment
    ASSIGN = "="

    COMMA = ","

    # Conditionals
    EQ = "=="
    NOT_EQ = "!="

    LT = "<"
    GT = ">"

    LTE = "<="
    GTE = ">="

    # Bracket types
    LBRACE = "{"
    RBRACE = "}"


T = TypeVar("T")


class Token(Generic[T]):
    def __init__(self, token_type: TokenType, value: T = None) -> None:
        """
        Class to represent a token used while lexing the source code.
        The token has a type, and an optional value.

        Here are the types of tokens:
        - INT: Integer literal -> 1, 2, 3, etc.
        - FLOAT: Float literal -> 1.84, 3.45, 3.1, etc.
        - STRING: String literal -> "Hello", "World", etc.
        - IDENTIFIER: Identifier -> a, b, c, etc (used for variables)
        - KEYWORD: Keyword -> if, else, while, etc.
        - OPERATOR: Operator -> +, -, *, etc.
        - PUNCTUATION: Punctuation -> (, ), {, }, etc.
        - EOF: End of file -> The end of the file.
        """
        self.token_type = token_type
        self.value = value

    def __repr__(self) -> str:
        return f"Token({self.token_type}{', value: ' + repr(self.value) if self.value else ''})"
