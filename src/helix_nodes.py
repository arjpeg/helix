from typing import Any

from .helix_token import Token, TokenType


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


class StringNode(ASTNode):
    def __init__(self, token: Token[str]):
        self.token = token
        self.value = token.value

    def __repr__(self):
        return f'"{self.value}"'


class ListNode(ASTNode):
    def __init__(self, elements: list[ASTNode]):
        self.elements = elements

    def __repr__(self):
        return f"List([{', '.join(map(str, self.elements))}])"


class DictNode(ASTNode):
    def __init__(self, elements: dict[Token[str], ASTNode]):
        self.elements = elements

    def __repr__(self):
        return (
            "Dict({\n"
            + ",\n".join(f"\t{key}: {value}" for key, value in self.elements.items())
            + "\n})"
        )


class IndexingNode(ASTNode):
    def __init__(self, list_node: Token[str], index: ASTNode):
        self.list_node = list_node
        self.index = index

    def __repr__(self):
        return f"{self.list_node.value}[{self.index}]"


class UnaryOpNode(ASTNode):
    def __init__(self, op: Token[str], expr: ASTNode):
        self.op = op
        self.expr = expr

    def __repr__(self):
        if self.op.token_type == TokenType.KEYWORD:
            return f"{self.op.value.value} {self.expr}"  # type: ignore

        return f"{self.op.token_type.value}{self.expr}"


class AssignNode(ASTNode):
    def __init__(self, name: Token[str], value: ASTNode):
        self.name = name
        self.value = value

    def __repr__(self):
        return f"AssignNode({self.name.value} = {self.value})"


class AssignIndexNode(ASTNode):
    def __init__(self, name: Token[str], index: ASTNode, value: ASTNode):
        self.name = name
        self.index = index
        self.value = value

    def __repr__(self):
        return f"AssignIndexNode({self.name.value}[{self.index}] = {self.value})"


class AssignPropertyNode(ASTNode):
    def __init__(self, name: Token[str], property: Token[str], value: ASTNode):
        self.name = name
        self.property = property
        self.value = value

    def __repr__(self):
        return f"AssignPropertyNode({self.name.value}.{self.property.value} = {self.value})"


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


class InNode(ASTNode):
    def __init__(self, left: ASTNode, right: ASTNode):
        self.left = left
        self.right = right

    def __repr__(self):
        return f"InNode({self.left} in {self.right})"


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


class ContinueNode(ASTNode):
    def __repr__(self):
        return "continue"


class BreakNode(ASTNode):
    def __repr__(self):
        return "break"


class FunctionExprNode(ASTNode):
    def __init__(self, arguments: list[Token[str]], body: ASTNode) -> None:
        """
        Used as an anonymous function.
        """

        self.arguments = arguments
        self.body = body

    def __repr__(self):
        return f"FunctionExprNode(({', '.join([arg.value for arg in self.arguments])}), {self.body})"


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


class PropertyAccessNode(ASTNode):
    def __init__(
        self,
        object: Token[str],
        property_lookups: list[Token[str] | "PropertyFunctionInvocationNode"],
    ):
        self.object = object
        self.property_lookups = property_lookups

    def __repr__(self):
        result = f"PropertyLookup({self.object.value}"

        for lookup in self.property_lookups:
            if isinstance(lookup, Token):
                result += f".{lookup.value}"

            else:
                result += f".{lookup}"

        return result + ")"


class PropertyFunctionInvocationNode(ASTNode):
    def __init__(self, identifier: Token[str], arguments: list[ASTNode]):
        self.identifier = identifier
        self.arguments = arguments

    def __repr__(self):
        return f"{self.identifier.value}({', '.join([str(arg) for arg in self.arguments])})"


class ReturnNode(ASTNode):
    def __init__(self, expr: ASTNode):
        self.expr = expr

    def __repr__(self):
        return f"ReturnNode({self.expr})"


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
