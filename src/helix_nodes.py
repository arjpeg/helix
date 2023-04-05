from typing import Any

from .helix_token import Token


class ASTNode:
    pass


class BinOpNode(ASTNode):
    def __init__(self, left: ASTNode, op: Token[str], right: ASTNode):
        self.left = left
        self.op = op
        self.right = right

    def __repr__(self):
        return f"({self.left} {self.op.token_type.value} {self.right})"


class VariableNode(ASTNode):
    def __init__(self, name: Token[str]):
        self.name = name

    def __repr__(self):
        return f"var({self.name.value})"


class NumberNode(ASTNode):
    def __init__(self, token: Token[int] | Token[float]):
        self.token = token
        self.value = token.value

    def __repr__(self):
        return str(self.value)


class UnaryOpNode(ASTNode):
    def __init__(self, op: Token[str], expr: ASTNode):
        self.op = op
        self.expr = expr

    def __repr__(self):
        return f"({self.op.token_type.value}{self.expr})"


class AssignNode(ASTNode):
    def __init__(self, name: Token[str], value: ASTNode):
        self.name = name
        self.value = value

    def __repr__(self):
        return f"AssignNode({self.name.value} = {self.value})"


class NoOpNode(ASTNode):
    def __repr__(self):
        return ""


class CompareNode(ASTNode):
    def __init__(self, left: ASTNode, op: Token[str], right: ASTNode):
        self.left = left
        self.op = op
        self.right = right

    def __repr__(self):
        return f"ConditionNode({self.left} {self.op.token_type.value} {self.right})"


class AndNode(ASTNode):
    def __init__(self, left: ASTNode, right: ASTNode):
        self.left = left
        self.right = right

    def __repr__(self):
        return f"AndNode({self.left}, {self.right})"


class OrNode(ASTNode):
    def __init__(self, left: ASTNode, right: ASTNode):
        self.left = left
        self.right = right

    def __repr__(self):
        return f"OrNode({self.left}, {self.right})"


class IfNode(ASTNode):
    def __init__(self, condition: CompareNode, body: ASTNode):
        self.condition = condition
        self.body = body

    def __repr__(self):
        return f"IfNode({self.condition}, {self.body})"


class ElseIfNode(ASTNode):
    def __init__(self, condition: CompareNode, body: ASTNode):
        self.condition = condition
        self.body = body

    def __repr__(self):
        return f"ElseIfNode({self.condition}, {self.body})"


class ElseNode(ASTNode):
    def __init__(self, body: ASTNode):
        self.body = body

    def __repr__(self):
        return f"ElseNode({self.body})"


class ConditionalStatementNode(ASTNode):
    def __init__(
        self, if_node: IfNode, elif_nodes: list[ElseIfNode], else_node: ElseNode | None
    ):
        """
        Represents a conditional statement, such as an if statement, part of an if-else statement, or part of an if-elif-else statement.
        """
        self.if_node = if_node
        self.elif_nodes = elif_nodes
        self.else_node = else_node

    def __repr__(self):
        res = f"({self.if_node}"

        for elif_node in self.elif_nodes:
            res += f"\n{elif_node}"

        if self.else_node:
            res += f"\n{self.else_node}"

        res += ")"

        return res


class ForNode(ASTNode):
    def __init__(self, identifier: Token[str], iterator: Any, body: ASTNode):
        self.identifier = identifier
        self.iterator = iterator
        self.body = body

    def __repr__(self):
        return f"ForNode({self.identifier.value} in {self.iterator}, {self.body})"


class WhileNode(ASTNode):
    def __init__(self, condition: CompareNode, body: ASTNode):
        self.condition = condition
        self.body = body

    def __repr__(self):
        return f"WhileNode({self.condition}, {self.body})"


class FunctionDefNode(ASTNode):
    def __init__(
        self, identifier: Token[str], arguments: list[Token[str]], body: ASTNode
    ) -> None:
        self.identifier = identifier
        self.arguments = arguments
        self.body = body

    def __repr__(self):
        return f"FunctionDefNode(fn {self.identifier.value}, ({', '.join([arg.value for arg in self.arguments])}), {self.body})"


class FunctionInvocationNode(ASTNode):
    def __init__(self, identifier: Token[str], arguments: list[ASTNode]) -> None:
        self.identifier = identifier
        self.arguments = arguments

    def __repr__(self):
        return f"{self.identifier.value}({', '.join([str(arg) for arg in self.arguments])})"


class BlockNode(ASTNode):
    def __init__(self, statements: list[ASTNode]):
        self.statements = statements

    def __str__(self):
        result = "Block {\n"

        for statement in self.statements:
            if isinstance(statement, NoOpNode):
                continue

            statement_str = str(statement).replace("\n", "\n\t")

            result += "\t" + statement_str + "\n"

        result += "}"

        return result
