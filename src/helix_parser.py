from typing import Any, Callable, Type, TypeVar

from src.utils.custom_iter import CustomIter

from .helix_nodes import *
from .helix_token import CONDITIONAL_OPERATORS, Keyword, Token, TokenType

T = TypeVar("T", IfNode, ElseIfNode)
V = TypeVar("V")


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
        Parse a list of statements.

        The grammar for this is:

        (statement)*
        """
        statements: list[ASTNode] = []

        while (
            self.current_token is not None
            and self.current_token.token_type != TokenType.EOF
        ):
            self.skip_newlines()

            # if the current token is a rbrace, then we've reached the end of the block
            if self.current_token.token_type == TokenType.RBRACE:
                break

            statements.append(self.statement())

        return BlockNode(statements)

    def statement(self) -> ASTNode:
        """
        Parse a single-line statement.

        The grammar for this is:
        statement : expr
            | for-stmt
            | while-stmt
            | if-stmt
            | func-def
            | assign-stmt
        """

        if self.current_token is None:
            return NoOpNode()

        # if the current token is an identifier, then it might be
        # a variable assignment, a function invocation, or a variable reference
        if self.current_token.token_type == TokenType.IDENTIFIER:
            self.advance()
            token = self.current_token
            self.rewind()

            if token and token.token_type == TokenType.LPAREN:
                return self.func_invocation()

            if token and token.token_type == TokenType.ASSIGN:
                return self.assign_stmt()

            return self.expr()            


        if self.current_token.token_type == TokenType.KEYWORD:
            if self.current_token.value == Keyword.LET:
                return self.assign_stmt()

            if self.current_token.value == Keyword.FOR:
                return self.for_stmt()

            elif self.current_token.value == Keyword.WHILE:
                return self.while_stmt()

            elif self.current_token.value == Keyword.IF:
                return self.if_stmt()

            elif self.current_token.value == Keyword.FN:
                return self.func_def()

        return self.expr()

    # region Statements
    def assign_stmt(self) -> ASTNode:
        """
        Parse an assignment. The grammar for this is:

        assign_stmt : (LET)? IDENTIFIER ASSIGN expr
        """
        if self.current_token.token_type == TokenType.KEYWORD:
            assert (
                self.current_token 
                and self.current_token.token_type == TokenType.KEYWORD
                and self.current_token.value == Keyword.LET            
            ), f"Expected 'let' in variable assignment, got {self.current_token}"

            self.advance() # skip the let keyword

        assert (
            self.current_token and self.current_token.token_type == TokenType.IDENTIFIER
        ), f"Expected identifier in variable assignment, got {self.current_token}"

        name = self.current_token

        self.advance()

        assert (
            self.current_token and self.current_token.token_type == TokenType.ASSIGN
        ), f"Expected '=' in variable assignment, got {self.current_token}"

        self.advance()

        value = self.expr()

        return AssignNode(name, value)

    def if_stmt(self) -> ASTNode:
        """
        Parse an if statement. The grammar for this is:

        if-stmt : IF compare-expr LBRACE (statement)* RBRACE (ELSE IF compare-expr LBRACE (statement)* RBRACE)* (ELSE LBRACE (statement)* RBRACE)?
        """
        assert (
            self.current_token and self.current_token.value == Keyword.IF
        ), f"Expected 'if' in if statement, got {self.current_token}"

        if_block = self._condition_op(IfNode)
        else_if_blocks: list[ElseIfNode] = []
        else_block: ElseNode | None = None

        self.skip_newlines()

        while (
            self.current_token
            and self.current_token.token_type == TokenType.KEYWORD
            and self.current_token.value == Keyword.ELSE
        ):
            # check if it's an else if
            self.advance()

            if (
                self.current_token
                and self.current_token.token_type == TokenType.KEYWORD
                and self.current_token.value == Keyword.IF
            ):
                else_if_blocks.append(self._condition_op(ElseIfNode))
                self.skip_newlines()

            else:
                if else_block is not None:
                    raise Exception("Cannot have more than one else block")

                assert (
                    self.current_token
                    and self.current_token.token_type == TokenType.LBRACE
                ), f"Expected '{{' in else block, got {self.current_token}"

                self.advance()

                else_block = ElseNode(self.statement_list())

                assert (
                    self.current_token
                    and self.current_token.token_type == TokenType.RBRACE
                ), f"Expected '}}' in else block, got {self.current_token}"

                self.advance()

        return ConditionalStatementNode(if_block, else_if_blocks, else_block)

    def while_stmt(self) -> ASTNode:
        """
        Parse a while loop. The grammar for this is:

        while-stmt : WHILE compare-expr LBRACE (statement)* RBRACE
        """
        assert (
            self.current_token
            and self.current_token.value == Keyword.WHILE
        ), "Expected 'while' in while loop"

        self.advance()

        condition = self.compare_expr()

        assert (
            self.current_token
            and self.current_token.token_type == TokenType.LBRACE
        ), "Expected '{{' in while loop"

        self.advance()

        body = self.statement_list()

        assert (
            self.current_token
            and self.current_token.token_type == TokenType.RBRACE
        ), "Expected '}}' in while loop"

        self.advance()

        return WhileNode(condition, body)

    def for_stmt(self) -> ASTNode:
        """
        Parse a for loop. The grammar for this is:

        for-stmt : FOR IDENTIFIER IN expr LBRACE (statement)* RBRACE
        """
        assert (
            self.current_token
            and self.current_token.value == Keyword.FOR
        ), "Expected 'for' in for loop"

        self.advance()

        assert (
            self.current_token
            and self.current_token.token_type == TokenType.IDENTIFIER
        ), "Expected identifier in for loop"

        token = self.current_token
        
        self.advance()

        assert (
            self.current_token
            and self.current_token.token_type == TokenType.KEYWORD
            and self.current_token.value == Keyword.IN
        ), "Expected 'in' in for loop after identifier"

        self.advance()

        expr = self.expr()

        assert (
            self.current_token
            and self.current_token.token_type == TokenType.LBRACE
        ), "Expected '{{' in for loop"

        self.advance()

        statements = self.statement_list()

        assert (
            self.current_token
            and self.current_token.token_type == TokenType.RBRACE
        ), "Expected '}}' in for loop"

        self.advance()

        return ForNode(token, expr, statements)

    # endregion

    def expr(self) -> ASTNode:
        """
        Parse an expression.

        The grammar for this is:

        expr : compare-expr (AND|OR compare-expr)*
        """
        compare_expr = self.compare_expr()

        while self.current_token and self.current_token.token_type == TokenType.KEYWORD:
            if self.current_token.value == Keyword.AND:
                self.advance()
                compare_expr = AndNode(compare_expr, self.compare_expr())

            elif self.current_token.value == Keyword.OR:
                self.advance()
                compare_expr = OrNode(compare_expr, self.compare_expr())

            else:
                break

        return compare_expr

    def compare_expr(self) -> ASTNode:
        """
        Parse a comparison expression.

        The grammar for this is:

        compare-expr : NOT compare-expr
             |  arith_expr ((EQ|NEQ|GT|LT|GTE|LTE) arith-expr)*
        """

        if (
            self.current_token
            and self.current_token.token_type == TokenType.KEYWORD
            and self.current_token.value == Keyword.NOT
        ):
            self.advance()

            return UnaryOpNode(self.current_token, self.compare_expr())

        res = self._bin_op(self.arith_expr, CONDITIONAL_OPERATORS, CompareNode)

        return res

    # region Math

    def arith_expr(self) -> ASTNode:
        """
        Parse an arithmetic expression.

        The grammar for this is:

        arith-expr : term ((PLUS|MINUS) term)*
        """

        return self._bin_op(self.term, [TokenType.PLUS, TokenType.MINUS])

    def term(self) -> ASTNode:
        """
        Parse a term.

        The grammar for this is:

        term : factor ((MUL|DIV) factor)*
        """
        return self._bin_op(self.factor, [TokenType.MUL, TokenType.DIV])

    def factor(self) -> ASTNode:
        """
        Parse a factor.

        The grammar for this is:

        factor : (PLUS|MINUS) factor
               | power
        """
        token = self.current_token

        assert token, "Expected factor, got EOF"

        if token.token_type in [TokenType.PLUS, TokenType.MINUS]:
            self.advance()
            return UnaryOpNode(token, self.factor())

        return self.power()

    def power(self) -> ASTNode:
        """
        Parse a power.

        The grammar for this is:

        power : atom (POW atom)*
        """
        return self._bin_op(self.atom, [TokenType.POW])

    def atom(self) -> ASTNode:
        """
        Parse an atom.

        The grammar for this is:

        atom : INT
             | FLOAT
             | LPAREN expr RPAREN
             | IDENTIFIER
        """
        token = self.current_token

        assert token, "Expected atom, got None"

        # Check if the token is EOF
        if token.token_type == TokenType.EOF:
            return NoOpNode()

        if token.token_type == TokenType.INT or token.token_type == TokenType.FLOAT:
            self.advance()
            return NumberNode(token)

        elif token.token_type == TokenType.LPAREN:
            self.advance()
            node = self.expr()
            assert self.current_token, "Expected ')'"
            assert (
                a := self.current_token.token_type
            ) == TokenType.RPAREN, "Expected ')', got " + str(a)
            self.advance()
            return node

        # the identifier is a variable
        elif token.token_type == TokenType.IDENTIFIER:
            token = self.current_token
            self.advance()

            return VariableNode(token)  # type: ignore

        raise Exception(
            f"Invalid syntax: {token}. Expected int, float, or identifier, got {token.token_type}"
        )

    # endregion

    # region Helper Functions
    @staticmethod
    def matches_type(token: Token[Any], token_types: list[TokenType]) -> bool:
        """
        Check if the token matches the given token types.
        """

        return token.token_type in token_types

    def skip_newlines(self) -> None:
        """
        Skip newlines.
        """

        while self.current_token and self.matches_type(
            self.current_token, [TokenType.NEWLINE]
        ):
            self.advance()

    def _bin_op(
        self,
        fn: Callable[[], ASTNode],
        ops: list[TokenType] | list[Keyword],
        return_type: Type[ASTNode] = BinOpNode,
    ) -> ASTNode:
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

            left = return_type(left, op, right)  # type: ignore

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
        assert (
            self.current_token
            and self.current_token.token_type == TokenType.KEYWORD
            and self.current_token.value == Keyword.IF
        ), "Expected 'if'"

        self.advance()

        condition = self.compare_expr()

        assert self.current_token, "Expected '{' after if condition"
        assert (
            a := self.current_token.token_type
        ) == TokenType.LBRACE, "Expected '{' after if condition, got " + str(a)

        self.advance()  # Consume lbrace
        self.advance()  # Consume newline

        statements = self.statement_list()

        assert self.current_token, "Expected '}' after if condition"
        assert (
            a := self.current_token.token_type
        ) == TokenType.RBRACE, "Expected '}' after if condition, got " + str(a)

        self.advance()  # Consume rbrace

        return return_type(condition, statements)  # type: ignore

    def advance(self) -> Token[Any] | None:
        """
        Advance the current token.
        """
        self.current_token = self.tokens.next()
        return self.current_token

    def rewind(self) -> Token[Any] | None:
        """
        Rewind the current token.
        """
        self.current_token = self.tokens.prev()
        return self.current_token

    # endregion
