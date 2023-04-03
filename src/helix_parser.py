from typing import Any, Callable, Type, TypeVar

from src.utils.custom_iter import CustomIter

from .helix_nodes import *
from .helix_token import Keyword, Token, TokenType

T = TypeVar("T", IfNode, ElseIfNode)


class Parser:
    def __init__(self, tokens: list[Token[Any]]):
        """
        Used to parse the tokens into an AST.
        """
        self.tokens = CustomIter(iter(tokens))
        self.current_token: Token[Any] | None = self.tokens.next()

    def parse(self) -> ASTNode:
        """
        Parse the tokens into an AST.
        """
        return self.statement_list()

    def statement_list(self) -> ASTNode:
        """
        Parse the tokens into an AST.

        Uses the following grammar:
        statement_list: statement (NEWLINE statement)*
        """
        statements: list[ASTNode] = []

        while (
            self.current_token is not None
            and self.current_token.token_type != TokenType.EOF
        ):
            if self.current_token and self.current_token.token_type == TokenType.RBRACE:
                # This is at the end of an if statement, or a function definition
                # Just return the statements we've parsed so far
                return BlockNode(statements)

            statement = self.statement()
            statements.append(statement)

            if (
                self.current_token is not None  # type: ignore
                and self.current_token.token_type == TokenType.NEWLINE
            ):
                self.advance()

        # Consume any trailing newline characters
        while (
            self.current_token is not None
            and self.current_token.token_type == TokenType.NEWLINE
        ):
            self.advance()

        # Return a NoOpNode if there are no statements
        if not statements:
            return NoOpNode()

        # Return a single statement if there's only one
        if len(statements) == 1:
            return statements[0]

        # Otherwise, create a sequence of statements
        return BlockNode(statements)

    def statement(self) -> ASTNode:
        """
        Parse a statement using the following grammar:
        statement: expr | assign_stmt | if_stmt
        """

        if (
            not self.current_token
            or self.current_token.token_type == TokenType.EOF
            or self.current_token.token_type == TokenType.NEWLINE
        ):
            return NoOpNode()

        if self.current_token.token_type == TokenType.IDENTIFIER:
            # this could be an assignment statement or a function call
            # check the next token to see which it is
            self.advance()

            if self.current_token.token_type == TokenType.ASSIGN:  # type: ignore
                # this is an assignment statement
                self.current_token = self.tokens.prev()
                return self.assign_stmt()

            if self.current_token.token_type == TokenType.LPAREN:  # type: ignore
                # this is a function call
                self.current_token = self.tokens.prev()
                return self.function_invocation()

            return NumberNode(self.tokens.prev().value)  # type: ignore

        if self.current_token.token_type == TokenType.KEYWORD:
            if self.current_token.value == Keyword.LET:
                return self.assign_stmt()

            if self.current_token.value == Keyword.IF:
                return self.if_stmt()

            if self.current_token.value == Keyword.FOR:
                return self.for_stmt()

            if self.current_token.value == Keyword.WHILE:
                return self.while_stmt()

            if self.current_token.value == Keyword.FUNCTION:
                return self.function_def()

        return self.expr()

    def assign_stmt(self) -> ASTNode:
        """
        Parse an assignment statement using the following grammar:
        assign_stmt: LET? IDENTIFIER EQUALS expr
        """
        if self.current_token and self.current_token.token_type == TokenType.KEYWORD:
            assert (
                self.current_token.value == Keyword.LET
            ), "Expected 'let' while parsing assignment statement"

            self.advance()

        token = self.current_token

        assert token, "Expected variable name while parsing assignment statement"
        assert (
            token.token_type == TokenType.IDENTIFIER
        ), f"Expected variable name while parsing assignment statement (got {token.token_type.value})"

        name = token.value

        self.advance()

        token = self.current_token

        assert token, "Expected '=' while parsing assignment statement"
        assert (
            token.token_type == TokenType.ASSIGN
        ), "Expected '=' while parsing assignment statement"

        self.advance()

        return AssignNode(name, self.expr())

    def if_stmt(self) -> ASTNode:
        """
        Parse an if statement using the following grammar:

        if_stmt : IF condition LBRACE NEWLINE statement_list RBRACE
        """
        if_node = self._condition_op(IfNode)
        else_if_nodes: list[ElseIfNode] = []
        else_node: ElseNode | None = None

        # skip over any newlines after the if statement
        self.skip_new_lines()

        # check for else if statements
        while self.current_token and self.current_token.token_type == TokenType.KEYWORD:
            self.skip_new_lines()

            if not self.current_token.value == Keyword.ELSE:
                break

            # this node could be an else if or an else
            self.advance()

            if (
                self.current_token
                and self.current_token.token_type == TokenType.KEYWORD
                and self.current_token.value == Keyword.IF
            ):
                # this is an else if
                else_if_nodes.append(self._condition_op(ElseIfNode))

                self.skip_new_lines()

            else:
                if else_node:
                    raise Exception("Cannot have more than one else statement")

                assert self.current_token, "Expected '{' while parsing else statement"
                assert (
                    self.current_token.token_type == TokenType.LBRACE
                ), "Expected '{' while parsing else statement, got " + str(
                    self.current_token
                )

                self.advance()  # consume the '{'
                self.advance()  # consume the newline

                statements = self.statement_list()

                assert (
                    self.current_token
                    and self.current_token.token_type == TokenType.RBRACE
                ), "Expected '}' while parsing else statement"

                self.advance()  # consume the '}'

                else_node = ElseNode(statements)

        return ConditionalStatementNode(if_node, else_if_nodes, else_node)

    def for_stmt(self) -> ASTNode:
        """
        Parse a for statement using the following grammar:

        for_stmt : FOR IDENTIFIER IN expr LBRACE NEWLINE statement_list NEWLINE RBRACE
        """
        self.advance()

        token = self.current_token

        assert (
            token and token.token_type == TokenType.IDENTIFIER
        ), "Expected variable name while parsing for statement"

        name = token.value

        self.advance()

        assert (
            self.current_token
            and self.current_token.token_type == TokenType.KEYWORD
            and self.current_token.value == Keyword.IN
        ), "Expected 'in' while parsing for statement, after variable name"

        self.advance()

        iterable = self.expr()

        assert (
            self.current_token and self.current_token.token_type == TokenType.LBRACE
        ), "Expected '{' while parsing for statement"

        self.advance()  # consume the '{'

        statements = self.statement_list()

        assert (
            self.current_token and self.current_token.token_type == TokenType.RBRACE
        ), "Expected '}' while parsing for statement after statement list"

        self.advance()  # consume the '}'

        return ForNode(name, iterable, statements)

    def while_stmt(self) -> ASTNode:
        """
        Parse a while statement using the following grammar:

        while_stmt : WHILE condition LBRACE NEWLINE statement_list NEWLINE RBRACE
        """
        self.advance()

        condition = self.condition()

        assert (
            self.current_token and self.current_token.token_type == TokenType.LBRACE
        ), "Expected '{' while parsing while statement"

        self.advance()  # consume the '{'

        statements = self.statement_list()

        assert (
            self.current_token and self.current_token.token_type == TokenType.RBRACE
        ), "Expected '}' while parsing while statement after statement list"

        self.advance()  # consume the '}'

        return WhileNode(condition, statements)

    def function_def(self) -> ASTNode:
        """
        Create a function definition using the following grammar:

        function_def : FUNCTION IDENTIFIER LPAREN (IDENTIFIER (COMMA IDENTIFIER)*)? RPAREN LBRACE NEWLINE statement_list NEWLINE RBRACE
        """
        self.advance()

        name_token = self.current_token

        assert (
            name_token and name_token.token_type == TokenType.IDENTIFIER
        ), "Expected function name while parsing function definition"

        name = name_token.value

        self.advance()

        assert (
            self.current_token and self.current_token.token_type == TokenType.LPAREN
        ), "Expected '(' while parsing function definition (for arguments)"

        self.advance()

        arg_identifiers: list[Token[str]] = []

        while self.current_token and self.current_token.token_type == TokenType.IDENTIFIER:  # type: ignore
            arg_identifiers.append(self.current_token)

            self.advance()

            if self.current_token and self.current_token.token_type == TokenType.COMMA:
                self.advance()

            elif (
                self.current_token and self.current_token.token_type == TokenType.RPAREN
            ):
                break

            else:
                raise Exception(
                    "Expected ',' or ')' while parsing function definition, got "
                    + str(self.current_token)
                )

        assert (
            self.current_token and self.current_token.token_type == TokenType.RPAREN
        ), "Expected ')' while parsing function definition (for arguments)"

        self.advance()

        assert (
            self.current_token and self.current_token.token_type == TokenType.LBRACE
        ), "Expected '{' while parsing function definition"

        self.advance()

        statements = self.statement_list()

        assert (
            self.current_token and self.current_token.token_type == TokenType.RBRACE
        ), "Expected '}' while parsing function definition after statement list"

        self.advance()

        return FunctionDefNode(name, arg_identifiers, statements)

    def function_invocation(self) -> ASTNode:
        """
        Create a function call using the following grammar:

        function_invocation : IDENTIFIER LPAREN (expr (COMMA expr)*)? RPAREN
        """
        name = self.current_token

        assert (
            name and name.token_type == TokenType.IDENTIFIER
        ), "Expected function name while parsing function invocation"

        self.advance()

        assert (
            self.current_token and self.current_token.token_type == TokenType.LPAREN
        ), "Expected '(' while parsing function invocation (for arguments)"

        self.advance()

        arg_exprs: list[ASTNode] = []

        while self.current_token and self.current_token.token_type != TokenType.RPAREN:  # type: ignore
            arg_exprs.append(self.expr())
            self.current_token = self.tokens.prev()

            assert (
                self.current_token and self.current_token.token_type != TokenType.EOF
            ), "Expected token while parsing function invocation"

            if self.current_token.token_type == TokenType.COMMA:  # type: ignore
                self.advance()

            elif self.current_token.token_type == TokenType.RPAREN:  # type: ignore
                break

            else:
                raise Exception(
                    "Expected ',' or ')' while parsing function invocation, got "
                    + str(self.current_token)
                )

        assert (
            self.current_token and self.current_token.token_type == TokenType.RPAREN
        ), "Expected ')' while parsing function invocation (for arguments)"

        self.advance()

        return FunctionInvocationNode(name, arg_exprs)

    def condition(self) -> ConditionNode:
        """
        Parse an if statement using the following grammar:

        condition : expr ((EQ | NEQ | LT | GT | LTE | GTE) expr)*
        """
        left = self.expr()

        while self.current_token and self.matches_type(
            self.current_token,
            [
                TokenType.EQ,
                TokenType.NOT_EQ,
                TokenType.LT,
                TokenType.GT,
                TokenType.LTE,
                TokenType.GTE,
            ],
        ):
            token = self.current_token
            self.advance()
            right = self.expr()

            left = ConditionNode(left, token, right)

        return left  # type: ignore

    def expr(self) -> ASTNode:
        """
        Parse an expression using the following grammar:
        expr: term ((PLUS | MINUS) term)*
        """
        return self._bin_op(
            self.term,
            [TokenType.PLUS, TokenType.MINUS],
        )

    def term(self) -> ASTNode:
        """
        Parse a term using the following grammar:
        term: factor ((MUL | DIV) factor)*
        """
        return self._bin_op(
            self.factor,
            [TokenType.MUL, TokenType.DIV],
        )

    def factor(self) -> ASTNode:
        """
        Parse a factor using the following grammar:
        factor: (PLUS | MINUS) factor | INTEGER | FLOAT | IDENTIFIER | LPAREN expr RPAREN
        """
        token = self.current_token

        assert token, "Expected token while parsing factor"

        if token.token_type in [TokenType.PLUS, TokenType.MINUS]:
            self.advance()
            return UnaryOpNode(token, self.factor())

        elif self.matches_type(token, [TokenType.INT, TokenType.FLOAT]):
            self.advance()
            return NumberNode(token)

        elif token.token_type == TokenType.IDENTIFIER:
            # Check if it's a function invocation
            if (
                self.advance()
                and self.current_token
                and self.current_token.token_type == TokenType.LPAREN
            ):
                self.current_token = self.tokens.prev()
                return self.function_invocation()

            # Otherwise, it's a variable
            self.advance()
            return NumberNode(token)

        elif token.token_type == TokenType.LPAREN:
            self.advance()
            node = self.expr()

            assert self.current_token, "Expected token while parsing factor"
            assert (
                self.current_token.token_type == TokenType.RPAREN
            ), f"Expected RPAREN, got {self.current_token}"

            self.advance()
            return node

        else:
            raise Exception(
                f"Invalid syntax while trying to parse a factor. Got: '{token}'"
            )

    def skip_new_lines(self) -> None:
        """
        Skip new lines.
        """
        while self.current_token and self.current_token.token_type == TokenType.NEWLINE:
            self.advance()

    @staticmethod
    def matches_type(token: Token[Any], token_types: list[TokenType]) -> bool:
        """
        Check if the token matches the given token types.
        """

        return token.token_type in token_types

    def _bin_op(self, fn: Callable[[], ASTNode], ops: list[TokenType]) -> ASTNode:
        """
        Helper function for parsing binary operations.
        """
        left = fn()

        while (
            self.current_token is not None
            and self.current_token.token_type != TokenType.EOF
            and self.current_token.token_type in ops
        ):
            op = self.current_token
            self.advance()
            right = fn()

            left = BinOpNode(left, op, right)

        return left

    def _condition_op(self, return_type: Type[T]) -> T:
        """
        Helper function for parsing conditional jumps. For example,

        ```python
        if x == 1:
            ...
        ```

        and

        ```python
        else if x == 2:
            ...
        ```

        both share similar grammar rules.
        """
        self.advance()

        condition = self.condition()

        print(condition)

        assert self.current_token, "Expected '{' after if condition"
        assert (
            a := self.current_token.token_type
        ) == TokenType.LBRACE, "Expected '{' after if condition, got " + str(a)

        self.advance()  # Consume newline

        statements = self.statement_list()

        assert self.current_token, "Expected '}' after if condition"
        assert (
            a := self.current_token.token_type
        ) == TokenType.RBRACE, "Expected '}' after if condition, got " + str(a)

        self.advance()  # Consume rbrace

        return return_type(condition, statements)

    def advance(self) -> Token[Any] | None:
        """
        Advance the current token.
        """
        self.current_token = self.tokens.next()
        return self.current_token
