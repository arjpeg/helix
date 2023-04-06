from typing import Generator

from .helix_token import Keyword, Token, TokenType

COMMENT = "#"
OPERATORS = [
    "+",
    "-",
    "*",
    "/",
    "%",
    "^",
    "(",
    ")",
    "=",
    "==",
    "!=",
    "<",
    ">",
    "<=",
    ">=",
    "{",
    "}",
    "[",
    "]",
    "!",
    ":",
]

CONDITIONAL_OPERATORS = ["=", "!", "<", ">", "<", ">"]

valid_token_types = (
    Token[str] | Token[int] | Token[float] | Token[Keyword] | Token[None]
)


class Lexer:
    def __init__(self, code: str) -> None:
        """
        Used to parse the source code into tokens.
        """
        self.iter = iter(code)
        self.current_char = next(self.iter, None)

    def advance(self) -> str | None:
        """
        Advance the current character.
        """
        self.current_char = next(self.iter, None)

        return self.current_char

    def generate_tokens(
        self,
    ) -> Generator[valid_token_types, None, None]:
        """
        Generate tokens from the source code.
        """
        while self.current_char is not None:
            if self.current_char == "\n":
                yield Token(TokenType.NEWLINE)
                self.advance()

            elif self.current_char.isspace():
                self.advance()

            elif self.current_char.isdigit():
                yield self.generate_number()

            elif self.current_char.isalpha():
                yield self.generate_identifier()

            elif self.current_char in ["'", '"']:
                yield self.generate_string()

            elif self.current_char == ",":
                yield Token(TokenType.COMMA)
                self.advance()

            elif self.current_char == "-":
                # this could be a minus operator or an arrow
                self.advance()

                if self.current_char == ">":  # type: ignore
                    yield Token(TokenType.ARROW)
                    self.advance()

                else:
                    yield Token(TokenType.MINUS)

            elif self.current_char in OPERATORS:
                if self.current_char in CONDITIONAL_OPERATORS:
                    yield self.generate_conditional_operator()
                else:
                    yield Token(TokenType(self.current_char))

                self.advance()

            elif self.current_char == COMMENT:
                self.skip_comment()

            elif self.current_char == ".":
                # this could be a float or a dot operator
                self.advance()

                if self.current_char and self.current_char.isdigit():
                    number = self.generate_number()

                    if number.token_type == TokenType.FLOAT:
                        # there were more than one decimal points
                        raise Exception(f"Invalid number: 0.{number.value}")

                    number = Token(TokenType.FLOAT, float(f"0.{number.value}"))
                    yield number
                else:
                    yield Token(TokenType.DOT)

            else:
                raise Exception(f"Invalid character: {repr(self.current_char)}")

        yield Token(TokenType.EOF)

    def generate_number(self) -> Token[int] | Token[float]:
        """
        Generate a number token.
        """
        number = ""

        while self.current_char is not None and self.current_char.isdigit():
            number += self.current_char
            self.advance()

        if self.current_char == ".":
            number += self.current_char
            self.current_char = self.advance()

            while self.current_char is not None and self.current_char.isdigit():
                number += self.current_char
                self.advance()

            return Token(TokenType.FLOAT, float(number))

        else:
            return Token[int](TokenType.INT, int(number))

    def generate_identifier(self) -> Token[str] | Token[Keyword]:
        """
        Generate an identifier token.
        """
        identifier = ""

        while self.current_char is not None and self.current_char.isalnum():
            identifier += self.current_char
            self.advance()

        if identifier in [kw.value for kw in Keyword]:
            return Token(TokenType.KEYWORD, Keyword(identifier))

        return Token(TokenType.IDENTIFIER, identifier)

    def generate_string(self) -> Token[str]:
        """
        Generate a string token.
        """
        string = ""
        quote_type = self.current_char or ""

        self.advance()  # Skip the first quote

        while self.current_char is not None and self.current_char != quote_type:
            string += self.current_char or ""
            self.advance()

        self.advance()  # Skip the last quote

        return Token[str](TokenType.STRING, string)

    def generate_conditional_operator(self) -> Token[None]:
        """
        Generate a conditional operator token.
        """
        operator = self.current_char or ""

        # handle = and ==
        if operator == "=":
            self.advance()

            if self.current_char == "=":
                self.advance()
                return Token(TokenType.EQ)

            else:
                return Token(TokenType.ASSIGN)

        # handle !=
        elif operator == "!":
            self.advance()

            if self.current_char == "=":
                self.advance()
                return Token(TokenType.NOT_EQ)

            else:
                raise Exception(f"Invalid operator: {operator}")

        # handle < and <=
        elif operator == "<":
            self.advance()

            if self.current_char == "=":
                self.advance()
                return Token(TokenType.LTE)

            else:
                return Token(TokenType.LT)

        # lastly, handle > and >=
        elif operator == ">":
            self.advance()

            if self.current_char == "=":
                self.advance()
                return Token(TokenType.GTE)

            else:
                return Token(TokenType.GT)

        raise SyntaxError(
            f"Unexpected character in conditional operator (expected: {CONDITIONAL_OPERATORS}, got: {operator})"
        )

    def skip_comment(self) -> None:
        """
        Skip a comment. (Move to the end of the line)
        """
        while self.current_char is not None and self.current_char != "\n":
            self.advance()
