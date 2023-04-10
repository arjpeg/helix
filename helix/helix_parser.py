from typing import Any, Callable, Type, TypeVar

from helix.utils.custom_iter import CustomIter

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
        # a variable assignment,  or a variable reference
        # the function call is handled in the expr function (atom)
        if self.current_token.token_type == TokenType.IDENTIFIER:
            self.advance()
            token = self.current_token
            self.rewind()

            if token and token.token_type == TokenType.ASSIGN:
                return self.assign_stmt()

            # fast forward and check if the next token is a lbracket
            self.advance()
            token = self.current_token
            self.rewind()

            if token and token.token_type == TokenType.LBRACKET:
                return self.assign_stmt()

            if token and token.token_type == TokenType.DOT:
                return self.assign_stmt()

            return self.expr()

        if self.current_token.token_type == TokenType.KEYWORD:
            if self.current_token.value in [Keyword.LET, Keyword.CONST]:
                return self.assign_stmt()

            if self.current_token.value == Keyword.FOR:
                return self.for_stmt()

            elif self.current_token.value == Keyword.WHILE:
                return self.while_stmt()

            elif self.current_token.value == Keyword.IF:
                return self.if_stmt()

            elif self.current_token.value == Keyword.FN:
                return self.func_def()

            elif self.current_token.value == Keyword.RETURN:
                return self.return_stmt()

            elif self.current_token.value == Keyword.BREAK:
                return self.break_stmt()

            elif self.current_token.value == Keyword.CONTINUE:
                return self.continue_stmt()

        return self.expr()

    # region Statements
    def assign_stmt(self) -> ASTNode:
        """
        Parse an assignment. The grammar for this is:

        assign-stmt : (LET)? IDENTIFIER ((LBRACKET expr RBRACKET)|DOT IDENTIFIER)? ASSIGN expr
        """

        is_const = False

        if self.current_token and self.current_token.token_type == TokenType.KEYWORD:
            assert (
                self.current_token
                and self.current_token.token_type == TokenType.KEYWORD
                and self.current_token.value in [Keyword.LET, Keyword.CONST]
            ), f"Expected 'let' or 'const' in variable assignment, got {self.current_token}"

            if self.current_token.value == Keyword.CONST:
                is_const = True

            self.advance()  # skip the assign keyword

        assert (
            self.current_token and self.current_token.token_type == TokenType.IDENTIFIER
        ), f"Expected identifier in variable assignment, got {self.current_token}"

        name = self.current_token
        index: ASTNode | None = None
        property_name: Token[Any] | None = None

        self.advance()

        if (
            self.current_token
            and self.current_token.token_type == TokenType.LBRACKET  # type: ignore
        ):
            # this is an index assignment
            self.advance()

            index = self.expr()

            assert (
                self.current_token
                and self.current_token.token_type == TokenType.RBRACKET
            ), f"Expected ']' in variable assignment, got {self.current_token}"

            self.advance()

        if (
            self.current_token
            and self.current_token.token_type == TokenType.DOT  # type: ignore
        ):
            # this is a property assignment
            self.advance()

            assert (
                self.current_token
                and self.current_token.token_type == TokenType.IDENTIFIER
            ), f"Expected identifier in variable assignment, got {self.current_token}"

            property_name = self.current_token

            self.advance()

            if (
                self.current_token
                and self.current_token.token_type != TokenType.ASSIGN  # type: ignore
            ):
                # this could either be a function call or a variable reference
                # check if the next token is a lparen
                if self.current_token.token_type == TokenType.LPAREN:
                    # this is a function call
                    self.advance()

                    return PropertyAccessNode(
                        name,
                        [
                            PropertyFunctionInvocationNode(
                                property_name,
                                self.sep_by(
                                    TokenType.COMMA, self.expr, stop=TokenType.RPAREN
                                ),
                            )
                        ],
                    )
                else:
                    # this is a variable reference
                    return PropertyAccessNode(name, [property_name])

        assert (
            self.current_token and self.current_token.token_type == TokenType.ASSIGN
        ), f"Expected '=' in variable assignment, got {self.current_token}"

        self.advance()
        value = self.expr()

        if index:
            return AssignIndexNode(name, index, value)

        if property_name:
            return AssignPropertyNode(name, property_name, value)

        return (
            AssignNode(name, value) if not is_const else AssignConstantNode(name, value)
        )

    def continue_stmt(self) -> ASTNode:
        self.advance()

        return ContinueNode()

    def break_stmt(self) -> ASTNode:
        self.advance()

        return BreakNode()

    def return_stmt(self) -> ASTNode:
        """
        Parse the return statement. The grammar for this is:

        return-stmt : RETURN expr
        """
        assert (
            self.current_token and self.current_token.value == Keyword.RETURN
        ), "Expected 'return' in return statement"

        self.advance()
        expr = self.expr()

        return ReturnNode(expr)

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
            self.current_token and self.current_token.value == Keyword.WHILE
        ), "Expected 'while' in while loop"

        self.advance()

        condition = self.compare_expr()

        assert (
            self.current_token and self.current_token.token_type == TokenType.LBRACE
        ), "Expected '{{' in while loop"

        self.advance()

        body = self.statement_list()

        assert (
            self.current_token and self.current_token.token_type == TokenType.RBRACE
        ), "Expected '}}' in while loop"

        self.advance()

        return WhileNode(condition, body)  # type: ignore

    def for_stmt(self) -> ASTNode:
        """
        Parse a for loop. The grammar for this is:

        for-stmt : FOR (LET)? IDENTIFIER IN expr LBRACE (statement)* RBRACE
        """
        assert (
            self.current_token and self.current_token.value == Keyword.FOR
        ), "Expected 'for' in for loop"

        self.advance()

        if self.current_token and self.current_token.value == Keyword.LET:
            self.advance()

        assert (
            self.current_token and self.current_token.token_type == TokenType.IDENTIFIER
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
            self.current_token and self.current_token.token_type == TokenType.LBRACE
        ), "Expected '{{' in for loop"

        self.advance()

        statements = self.statement_list()

        assert (
            self.current_token and self.current_token.token_type == TokenType.RBRACE
        ), "Expected '}}' in for loop"

        self.advance()

        return ForNode(token, expr, statements)

    def func_def(self) -> ASTNode:
        """
        Parse a function definition. The grammar for this is:

        func-def : FUNC IDENTIFIER LPAREN (IDENTIFIER (COMMA IDENTIFIER)*)? RPAREN LBRACE (statement)* RBRACE
        """
        assert self.current_token and self.current_token.value == Keyword.FN

        self.advance()

        assert (
            self.current_token and self.current_token.token_type == TokenType.IDENTIFIER
        )

        fn_name = self.current_token
        params: list[Token[Any]] = []

        self.advance()

        # check for any '('
        if self.current_token.token_type == TokenType.LPAREN:  # type: ignore
            # check for parameters
            self.advance()

            params = self.sep_by(
                TokenType.COMMA,
                self.parse_identifier,
                TokenType.RPAREN,
            )

        # find the lbrace
        assert self.current_token and self.current_token.token_type == TokenType.LBRACE

        self.advance()

        body = self.statement_list()

        assert self.current_token and self.current_token.token_type == TokenType.RBRACE

        self.advance()

        return FunctionDefNode(fn_name, params, body)

    # endregion

    # region Expressions
    def expr(self) -> ASTNode:
        """
        Parse an expression.

        The grammar for this is:

        expr : compare-expr (AND|OR compare-expr)*
             | func-expr
        """
        if self.current_token and self.current_token.token_type == TokenType.LPAREN:
            # check for a function expr
            self.advance()
            times_advanced = 1
            is_fn_def = True

            params: list[Token[str]] = []

            while self.current_token:
                if self.current_token.token_type == TokenType.RPAREN:  # type: ignore
                    break

                if self.current_token.token_type == TokenType.COMMA:  # type: ignore
                    pass
                elif self.current_token.token_type == TokenType.IDENTIFIER:  # type: ignore
                    params.append(self.current_token)

                else:  # this wasn't a function expr, so we need to rewind
                    for _ in range(times_advanced):
                        self.rewind()

                    is_fn_def = False

                    break

                self.advance()
                times_advanced += 1

            if is_fn_def:
                assert (
                    self.current_token
                    and self.current_token.token_type == TokenType.RPAREN
                )

                self.advance()

                if not self.current_token.token_type == TokenType.ARROW:  # type: ignore
                    # this is a tuple
                    return TupleNode([VariableNode(node) for node in params])

                assert (
                    self.current_token
                    and self.current_token.token_type == TokenType.ARROW
                )

                self.advance()

                if self.current_token.token_type == TokenType.LBRACE:
                    self.advance()

                    body = self.statement_list()

                    assert (
                        self.current_token
                        and self.current_token.token_type == TokenType.RBRACE
                    ), "Expected '}}' in function expression"

                    self.advance()

                    return FunctionExprNode(params, body)

                else:
                    # this is a single expression
                    expr = self.expr()

                    return FunctionExprNode(params, ReturnNode(expr))

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
             |  arith_expr (IN arith-expr)?
        """

        if self.current_token and self.current_token.value == Keyword.NOT:
            tok = self.current_token
            self.advance()
            return UnaryOpNode(tok, self.compare_expr())

        arith_expr = self.arith_expr()

        if self.current_token and self.current_token.token_type == TokenType.KEYWORD:
            if self.current_token.value == Keyword.IN:
                self.advance()
                return InNode(arith_expr, self.arith_expr())
            else:
                raise SyntaxError(
                    f"Unexpected keyword {self.current_token.value} in compare expression"
                )

        while (
            self.current_token is not None
            and self.current_token.token_type != TokenType.EOF
            and self.current_token.token_type in CONDITIONAL_OPERATORS
        ):
            op = self.current_token
            self.advance()
            right = self.arith_expr()

            arith_expr = CompareNode(arith_expr, op, right)

        return arith_expr

    # endregion

    # region Data Structures
    def generate_list(self) -> ASTNode:
        """
        Generate a list. The grammar for this is:

        list : LBRACKET (expr (COMMA expr)*)? RBRACKET
        """
        assert (
            self.current_token and self.current_token.token_type == TokenType.LBRACKET
        ), "Expected '[' to start list"

        self.advance()

        elements: list[ASTNode] = []

        while self.current_token:
            self.skip_newlines()

            if self.current_token.token_type == TokenType.RBRACKET:  # type: ignore
                break

            elements.append(self.expr())

            if self.current_token and self.current_token.token_type == TokenType.COMMA:  # type: ignore
                self.advance()
            elif self.current_token.token_type not in [TokenType.NEWLINE, TokenType.RBRACKET]:  # type: ignore
                raise SyntaxError("Expected ',' to seperate items in list")

        assert (
            self.current_token and self.current_token.token_type == TokenType.RBRACKET
        ), "Expected ']' to end list"
        self.advance()

        return ListNode(elements)

    def generate_dict(self) -> ASTNode:
        """
        Generate a dictionary. The grammar for this is:

        dict : LBRACE (IDENTIFIER COLON expr (COMMA IDENTIFIER COLON expr)*)? RBRACE
        """
        assert (
            self.current_token and self.current_token.token_type == TokenType.LBRACE
        ), "Expected '{' to start dict"

        self.advance()
        values = {}

        while self.current_token:
            self.skip_newlines()

            if self.current_token.token_type == TokenType.RBRACE:  # type: ignore
                break

            key = self.current_token

            assert (
                key and key.token_type == TokenType.IDENTIFIER
            ), "Expected identifier as key in dict"

            self.advance()

            assert (
                self.current_token and self.current_token.token_type == TokenType.COLON
            ), "Expected ':' to seperate key and value in dict"

            self.advance()

            value = self.expr()

            values[key.value] = value

            if self.current_token and self.current_token.token_type == TokenType.COMMA:  # type: ignore
                self.advance()

            elif self.current_token.token_type not in [TokenType.NEWLINE, TokenType.RBRACE]:  # type: ignore
                raise SyntaxError("Expected ',' to seperate items in dict")

        assert (
            self.current_token and self.current_token.token_type == TokenType.RBRACE
        ), "Expected '}' to end dict"

        self.advance()

        return DictNode(values)

    def generate_tuple(self) -> ASTNode:
        """
        Generate a tuple. The grammar for this is:

        tuple : LPAREN (expr (COMMA expr)*)? RPAREN
        """
        values: list[ASTNode] = []

        assert (
            self.current_token and self.current_token.token_type == TokenType.LPAREN
        ), "Expected '(' to start tuple"

        self.advance()

        while self.current_token:
            self.skip_newlines()

            if self.current_token.token_type == TokenType.RPAREN:  # type: ignore
                break

            values.append(self.expr())

            if self.current_token.token_type == TokenType.RPAREN:  # type: ignore
                break

            assert (
                self.current_token and self.current_token.token_type == TokenType.COMMA
            ), f"Expected ',' to seperate items in tuple, got {self.current_token}"

            self.advance()

        assert (
            self.current_token and self.current_token.token_type == TokenType.RPAREN
        ), "Expected ')' to end tuple"

        self.advance()

        # if there was only one value, then it might be a parenthesized expression
        # so we return the value instead of a tuple
        if len(values) == 1:
            return values[0]

        return TupleNode(values)

    # endregion

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
             | STRING
             | list
             | dict
             | tuple
             | IDENTIFIER
             | IDENTIFIER LPAREN (expr (COMMA expr)*)? RPAREN
             | IDENTIFIER LBRACKET expr RBRACKET
             | IDENTIFIER (DOT IDENTIFIER (LPAREN expr (COMMA expr)* RPAREN)?)*
        """
        token = self.current_token

        assert token, "Expected atom, got None"

        # Check if the token is EOF
        if token.token_type == TokenType.EOF:
            return NoOpNode()

        if token.token_type == TokenType.INT or token.token_type == TokenType.FLOAT:
            self.advance()
            return NumberNode(token)

        elif token.token_type == TokenType.STRING:
            self.advance()
            return StringNode(token)

        elif token.token_type == TokenType.LBRACKET:
            return self.generate_list()

        elif token.token_type == TokenType.LBRACE:
            return self.generate_dict()

        elif token.token_type == TokenType.LPAREN:
            return self.generate_tuple()

        # the identifier is either a variable, function call, or list/dict access
        elif token.token_type == TokenType.IDENTIFIER:
            token = self.current_token
            self.advance()

            if self.current_token and self.current_token.token_type == TokenType.LPAREN:
                self.advance()

                params_list: list[ASTNode] = []

                while self.current_token:
                    if self.current_token.token_type == TokenType.RPAREN:  # type: ignore
                        break

                    params_list.append(self.expr())

                    if self.current_token.token_type == TokenType.COMMA:  # type: ignore
                        self.advance()

                assert (
                    self.current_token
                    and self.current_token.token_type == TokenType.RPAREN
                ), "Expected ')'"
                self.advance()

                return FunctionInvocationNode(token, params_list)  # type: ignore

            # check if it's a list access
            elif (
                self.current_token
                and self.current_token.token_type == TokenType.LBRACKET
            ):
                self.advance()
                index = self.expr()

                assert (
                    self.current_token
                    and self.current_token.token_type == TokenType.RBRACKET
                ), "Expected ']'"

                self.advance()

                return IndexingNode(token, index)  # type: ignore

            # check if it's a property access
            elif self.current_token and self.current_token.token_type == TokenType.DOT:
                properties: list[Token[Any] | PropertyFunctionInvocationNode] = []
                object = token

                while (
                    self.current_token
                    and self.current_token.token_type == TokenType.DOT
                ):
                    # check for function call, e.g. foo.bar()
                    self.advance()

                    if self.current_token.token_type != TokenType.IDENTIFIER:  # type: ignore
                        raise Exception("Expected identifier after '.'")

                    self.advance()
                    token = self.current_token
                    self.rewind()

                    if token.token_type == TokenType.LPAREN:
                        # function call
                        token = self.current_token

                        self.advance()
                        self.advance()

                        args: list[ASTNode] = []

                        while self.current_token:
                            if self.current_token.token_type == TokenType.RPAREN:
                                break

                            self.skip_newlines()
                            args.append(self.expr())
                            self.skip_newlines()

                            if self.current_token.token_type == TokenType.COMMA:
                                self.advance()

                        assert (
                            self.current_token
                            and self.current_token.token_type == TokenType.RPAREN
                        ), "Expected ')'"

                        self.advance()

                        properties.append(PropertyFunctionInvocationNode(token, args))

                    else:
                        # property access
                        properties.append(self.current_token)
                        self.advance()

                return PropertyAccessNode(object, properties)  # type: ignore

            return VariableNode(token)  # type: ignore

        raise Exception(
            f"Invalid syntax: {token}. Expected int, float, string, or identifier, got {token.token_type}"
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

    def peek(self) -> Token[Any] | None:
        """
        Peek at the next token.
        """
        self.advance()
        tok = self.current_token
        self.rewind()

        return tok

    def parse_identifier(self) -> Token[str]:
        assert (
            self.current_token and self.current_token.token_type == TokenType.IDENTIFIER
        )

        token = self.current_token
        self.advance()

        return token

    def sep_by(self, token: TokenType, fn: Callable[[], V], stop: TokenType) -> list[V]:
        """
        Parse a list of expressions separated by a token.
        """

        exprs: list[V] = []

        while self.current_token and self.current_token.token_type != stop:
            exprs.append(fn())

            if self.current_token and self.current_token.token_type == token:
                self.advance()

        self.advance()

        return exprs

    # endregion
