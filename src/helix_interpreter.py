from src.helix_token import TokenType

from .helix_nodes import *


class Interpreter:
    def __init__(self, ast: ASTNode) -> None:
        self.ast = ast
        self.variables: dict[str, float] = {}

    def interpret(self) -> None:
        self.visit(self.ast)

    def visit(self, node: ASTNode) -> float:
        # Get the name of the method to call
        method_name = f"visit_{type(node).__name__}"

        # Get the method from 'self' object
        method = getattr(self, method_name, self.generic_visit)

        # Call the method as we return it
        return method(node)  # type: ignore

    def generic_visit(self, node: ASTNode) -> None:
        raise Exception(f"No visit_{type(node).__name__} method defined")

    def visit_NumberNode(self, node: NumberNode) -> float:
        return node.value

    def visit_BinOpNode(self, node: BinOpNode) -> float:
        if node.op.token_type == TokenType.PLUS:
            return self.visit(node.left) + self.visit(node.right)

        elif node.op.token_type == TokenType.MINUS:
            return self.visit(node.left) - self.visit(node.right)

        elif node.op.token_type == TokenType.MUL:
            return self.visit(node.left) * self.visit(node.right)

        elif node.op.token_type == TokenType.DIV:
            return self.visit(node.left) / self.visit(node.right)

        return 0

    def visit_UnaryOpNode(self, node: UnaryOpNode) -> float:
        if node.op.token_type == TokenType.PLUS:
            return +self.visit(node.expr)
        elif node.op.token_type == TokenType.MINUS:
            return -self.visit(node.expr)
        return 0

    def visit_AssignNode(self, node: AssignNode) -> float:
        self.variables[node.name.value] = self.visit(node.value)
        return self.variables[node.name.value]

    def visit_ConditionNode(self, node: CompareNode) -> float:
        if node.op.token_type == TokenType.EQ:
            return self.visit(node.left) == self.visit(node.right)
        elif node.op.token_type == TokenType.NOT_EQ:
            return self.visit(node.left) != self.visit(node.right)
        elif node.op.token_type == TokenType.LT:
            return self.visit(node.left) < self.visit(node.right)
        elif node.op.token_type == TokenType.LTE:
            return self.visit(node.left) <= self.visit(node.right)
        elif node.op.token_type == TokenType.GT:
            return self.visit(node.left) > self.visit(node.right)
        elif node.op.token_type == TokenType.GTE:
            return self.visit(node.left) >= self.visit(node.right)
        return 0

    def visit_IfNode(self, node: IfNode) -> float:
        if self.visit(node.condition):
            return self.visit(node.body)

        return 0

    def visit_SequenceNode(self, node: BlockNode) -> float:
        for child in node.statements:
            self.visit(child)

        return 0
