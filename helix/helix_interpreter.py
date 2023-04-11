from helix.helix_nodes import *
from helix.helix_symbol_table import SymbolTable
from helix.helix_token import Keyword
from helix.helix_values import *


def custom_print(*args: Any):
    for arg in args:
        if isinstance(arg, String):
            print(arg.value, end=" ")
        else:
            print(arg, end=" ")

    print()


def custom_input(prompt: String | None = None):
    if prompt:
        print(prompt.value, end="")

    return String(input())


global_symbol_table = SymbolTable(
    {
        "print": BuiltInFunction("print", custom_print),
        "input": BuiltInFunction("input", custom_input),
    }
)


class Interpreter:
    def __init__(self) -> None:
        self.symbol_table = global_symbol_table

    def visit(self, node: ASTNode):
        method_name = f"visit_{type(node).__name__}"
        method = getattr(self, method_name, self.no_visit_method)

        return method(node)

    def no_visit_method(self, node: ASTNode):
        raise Exception(f"No visit_{type(node).__name__} method defined")

    def visit_NoOpNode(self, _: NoOpNode):
        pass

    def visit_StringNode(self, node: StringNode):
        return String(node.token.value)

    def visit_ListNode(self, node: ListNode):
        return List([self.visit(element) for element in node.elements])

    def visit_DictNode(self, node: DictNode):
        return Dict({key: self.visit(value) for key, value in node.elements.items()})

    def visit_TupleNode(self, node: TupleNode):
        return Tuple([self.visit(element) for element in node.elements])

    def visit_IndexingNode(self, node: IndexingNode):
        list_node = self.visit(VariableNode(node.list_node))
        index = self.visit(node.index)

        return list_node.index(index)

    def visit_BlockNode(self, node: BlockNode):
        for child in node.statements:
            self.visit(child)

    def visit_NumberNode(self, node: NumberNode) -> Number:
        return Number(node.token.value)

    def visit_UnaryOpNode(self, node: UnaryOpNode):
        expr = self.visit(node.expr)

        if node.op.token_type == TokenType.MINUS:
            return expr.mul(Number(-1))

        if node.op.value == Keyword.NOT:
            return Boolean(not expr.value)

        return expr

    def visit_BinOpNode(self, node: BinOpNode):
        left = self.visit(node.left)
        right = self.visit(node.right)

        if node.op.token_type == TokenType.PLUS:
            return left.add(right)
        elif node.op.token_type == TokenType.MINUS:
            return left.sub(right)
        elif node.op.token_type == TokenType.MUL:
            return left.mul(right)
        elif node.op.token_type == TokenType.DIV:
            return left.div(right)
        elif node.op.token_type == TokenType.POW:
            return left.pow(right)

    def visit_AssignNode(self, node: AssignNode):
        name = node.name.value
        value = self.visit(node.value)

        if self.symbol_table.get(name) == None:
            # this is a new variable
            self.symbol_table.set(name, value)
        else:
            self.symbol_table.update(name, value)

    def visit_AssignConstantNode(self, node: AssignConstantNode):
        name = node.name
        value = self.visit(node.value)

        self.symbol_table.set(name.value, value, True)

    def visit_AssignPropertyNode(self, node: AssignPropertyNode):
        name = self.visit(VariableNode(node.name))
        value = self.visit(node.value)

        name.set_property(node.property.value, value)

        return value

    def visit_CompareNode(self, node: CompareNode) -> Boolean:
        left = self.visit(node.left)
        right = self.visit(node.right)

        try:
            if node.op.token_type == TokenType.EQ:
                return left.equals(right)

            if node.op.token_type == TokenType.NOT_EQ:
                return Boolean(not left.equals(right).value)

            if node.op.token_type == TokenType.GT:
                return left.greater_than(right)

            if node.op.token_type == TokenType.GTE:
                return left.greater_than_equals(right)

            if node.op.token_type == TokenType.LT:
                return left.less_than(right)

            if node.op.token_type == TokenType.LTE:
                return left.less_than_equals(right)

        except Exception:
            return Boolean(False)

        return Boolean(False)

    def visit_InNode(self, node: InNode):
        left = self.visit(node.left)
        right = self.visit(node.right)

        return right.contains(left)

    def visit_AndNode(self, node: AndNode):
        left_cond = self.visit(node.left)
        right_cond = self.visit(node.right)

        return Boolean(left_cond.value and right_cond.value)

    def visit_OrNode(self, node: OrNode):
        left_cond = self.visit(node.left)
        right_cond = self.visit(node.right)

        return Boolean(left_cond.value or right_cond.value)

    def visit_ConditionalStatementNode(self, node: ConditionalStatementNode):
        if_node_succesful = self.visit_IfNode(node.if_node)

        if if_node_succesful.value:
            return if_node_succesful

        for elif_node in node.elif_nodes:
            elif_node_succesful = self.visit_ElseIfNode(elif_node)

            if elif_node_succesful.value:
                return elif_node_succesful

        if node.else_node is not None:
            return self.visit_ElseNode(node.else_node)

    def visit_IfNode(self, node: IfNode):
        cond = self.visit(node.condition)

        if cond.value:
            self.visit(node.body)

        return cond

    def visit_ElseIfNode(self, node: ElseIfNode):
        cond = self.visit(node.condition)

        if cond.value:
            self.visit(node.body)

        return cond

    def visit_ElseNode(self, node: ElseNode):
        self.visit(node.body)

        return Boolean(True)

    def visit_ForNode(self, node: ForNode):
        self.symbol_table.push_scope()

        variable_name = node.identifier.value
        iterator = self.visit(node.iterator)

        if not getattr(iterator, "iter", None):
            raise Exception(f"{iterator} is not iterable")

        for value in iterator.iter():
            self.symbol_table.set(variable_name, value)
            self.visit(node.body)

        self.symbol_table.pop_scope()

    def visit_WhileNode(self, node: WhileNode):
        self.symbol_table.push_scope()

        while self.visit(node.condition).value:
            self.visit(node.body)

        self.symbol_table.pop_scope()

    def visit_FunctionDefNode(self, node: FunctionDefNode):
        name = node.identifier.value
        params = node.arguments
        body = node.body

        function = Function(name, [param.value for param in params], body)  # type: ignore

        self.symbol_table.set(name, function)

    def visit_FunctionExprNode(self, node: FunctionExprNode):
        # used to create an anonymous function
        params = node.arguments
        body = node.body

        return Function("anonymous", [param.value for param in params], body)  # type: ignore

    def visit_FunctionInvocationNode(self, node: FunctionInvocationNode):
        name = node.identifier.value
        args = [self.visit(arg) for arg in node.arguments]

        function: Function | None = self.symbol_table.get(name)

        if function is None:
            raise Exception(f"Function {name} is not defined")

        return function.call(args, self.symbol_table, self.visit)

    def visit_ReturnNode(self, node: ReturnNode):
        return self.visit(node.expr)

    def visit_VariableNode(self, node: VariableNode):
        name = node.name.value
        value = self.symbol_table.get(name)

        if value is None:
            raise Exception(f"Variable {name} is not defined")

        return value

    def visit_PropertyAccessNode(self, node: PropertyAccessNode):
        # the property accesses are chained, so we need to visit each one
        # and get the value of the property
        # value = self.visit(VariableNode(node.object))

        # for property in node.property_lookups:
        #     if isinstance(property, PropertyFunctionInvocationNode):
        #         # if the property is a function invocation, such as
        #         # a.print(), then we need to execute the function

        #         # property = value.get_property(property.name.value)
        print(node)
