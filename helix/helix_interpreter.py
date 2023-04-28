from helix.builtins import GLOBAL_SYMBOL_TABLE
from helix.data.boolean import Boolean
from helix.data.dict import Dict
from helix.data.function import Function
from helix.data.list import List
from helix.data.number import Number
from helix.data.string import String
from helix.data.tuple import Tuple
from helix.helix_context import Context
from helix.helix_nodes import *
from helix.helix_token import Keyword

# from helix.data


class Interpreter:
    def __init__(self, context: Context | None = None) -> None:
        self.context = context if context else Context(GLOBAL_SYMBOL_TABLE)

    def visit(self, node: ASTNode):
        method_name = f"visit_{type(node).__name__}"
        method = getattr(self, method_name, self.no_visit_method)

        return method(node)

    def no_visit_method(self, node: ASTNode):
        raise Exception(f"No visit_{type(node).__name__} method defined")

    def visit_NoOpNode(self, _: NoOpNode):
        pass

    # region Data types
    def visit_StringNode(self, node: StringNode):
        return String(node.token.value)

    def visit_ListNode(self, node: ListNode):
        return List([self.visit(element) for element in node.elements])

    def visit_DictNode(self, node: DictNode):
        return Dict({key: self.visit(value) for key, value in node.elements.items()})

    def visit_TupleNode(self, node: TupleNode):
        return Tuple([self.visit(element) for element in node.elements])

    # endregion

    def visit_IndexingNode(self, node: IndexingNode):
        list_node = self.visit(VariableNode(node.list_node))
        index = self.visit(node.index)

        return list_node.index(index)

    def visit_BlockNode(self, node: BlockNode):
        for child in node.statements:
            self.visit(child)

            fn_stack = self.context.fn_context_stack

            if len(fn_stack) and fn_stack[-1]["should_return"]:
                print(f"\n{node}")
                print(child)
                print(fn_stack[-1])
                input("stopping from visit_BlockNode")
                break

    # region Math

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
        print("expr:", node)
        print("left:", node.left, "type:", type(node.left))
        print("right:", node.right, "type:", type(node.right))
        input("in visit_BinOpNode")
        print()

        left = self.visit(node.left)
        right = self.visit(node.right)

        print("in expr,", node)
        print("left:", left, "type:", type(left))
        print("right:", right, "type:", type(right))
        input("finished evaluating left and right")

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

    # endregion

    # region Variables

    def visit_NewAssignNode(self, node: NewAssignNode):
        self.context.in_var_declaration = True

        name = node.name.value
        value = self.visit(node.value)

        self.context.symbol_table.set(name, value)

        self.context.in_var_declaration = False

    def visit_ReAssignNode(self, node: ReAssignNode):
        self.context.in_var_declaration = True

        name = node.name.value
        value = self.visit(node.value)

        # make sure the variable exists
        if self.context.symbol_table.get(name) is None:
            raise Exception(f"Variable {name} is not defined")

        self.context.symbol_table.update(name, value)

        self.context.in_var_declaration = False

    def visit_AssignConstantNode(self, node: AssignConstantNode):
        self.context.in_var_declaration = True

        name = node.name
        value = self.visit(node.value)

        self.context.symbol_table.set(name.value, value, True)

        self.context.in_var_declaration = False

    def visit_AssignPropertyNode(self, node: AssignPropertyNode):
        self.context.in_var_declaration = True

        name = self.visit(VariableNode(node.name))
        value = self.visit(node.value)

        name.set_property(node.property.value, value)

        self.context.in_var_declaration = False

        return value

    def visit_VariableNode(self, node: VariableNode):
        name = node.name.value
        value = self.context.symbol_table.get(name)

        if value is None:
            raise Exception(f"Variable {name} is not defined")

        return value

    def visit_PropertyAccessNode(self, node: PropertyAccessNode):
        # the property accesses are chained, so we need to visit each one
        # and get the value of the property
        value = self.visit(VariableNode(node.object))

        for property_lookup in node.property_lookups:
            if isinstance(property_lookup, Token):
                value = value.get_property(property_lookup.value)

            else:
                # this is a function call

                # get the name of the function
                function_name = property_lookup.identifier.value

                # get the function
                function: Function = value.get_property(function_name)

                # get the arguments
                args = [self.visit(arg) for arg in property_lookup.arguments]

                # call the function
                value = function.call(args, self.context, self.visit)

        return value

    # endregion

    # region Control Flow
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

        if self.context.get_fn_context()["should_return"]:
            # print(self.context.return_value)
            input("should return --- conditional statement")
            return

        input("if node not succesful - evaluating elif nodes")

        for elif_node in node.elif_nodes:
            elif_node_succesful = self.visit_ElseIfNode(elif_node)

            if elif_node_succesful.value:
                return elif_node_succesful

            if self.context.get_fn_context()["should_return"]:
                return

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

    # endregion

    # region Loops
    def visit_ForNode(self, node: ForNode):
        self.context.symbol_table.push_scope()

        variable_name = node.identifier.value
        iterator = self.visit(node.iterator)

        if not getattr(iterator, "iter", None):
            raise Exception(f"{iterator} is not iterable")

        for value in iterator.iter():
            self.context.symbol_table.set(variable_name, value)
            self.visit(node.body)

        self.context.symbol_table.pop_scope()

    def visit_WhileNode(self, node: WhileNode):
        self.context.symbol_table.push_scope()

        while self.visit(node.condition).value:
            self.visit(node.body)

        self.context.symbol_table.pop_scope()

    # endregion

    # region Functions
    def visit_FunctionDefNode(self, node: FunctionDefNode):
        name = node.identifier.value
        params = node.arguments
        body = node.body

        function = Function(name, [param.value for param in params], body)  # type: ignore

        self.context.symbol_table.set(name, function)

    def visit_FunctionExprNode(self, node: FunctionExprNode):
        # used to create an anonymous function
        params = node.arguments
        body = node.body

        return Function("anonymous", [param.value for param in params], body)  # type: ignore

    def visit_FunctionInvocationNode(self, node: FunctionInvocationNode):
        name = node.identifier.value
        args = [self.visit(arg) for arg in node.arguments]

        function: Function | None = self.context.symbol_table.get(name)

        if function is None:
            raise Exception(f"Function {name} is not defined")

        self.context.push_fn_context(function.name)

        print("\n")
        print(function)
        print(args)
        print(self.context.fn_context_stack)
        print(self.context.get_fn_context())
        input("in function invocation")
        print()

        res = function.call(args, self.context, self.visit)

        print("\n")
        print(res, "context:", self.context.fn_context_stack)
        print(function)
        input("after function invocation")

        self.context.pop_fn_context()

        return res

    def visit_ReturnNode(self, node: ReturnNode):
        self.context.get_fn_context()["return_value"] = self.visit(node.expr)
        self.context.get_fn_context()["should_return"] = True

        print(self.context.get_fn_context()["return_value"])
        print(self.context.fn_context_stack)
        input("return node")

        # No need to return anything here, the return value is stored in the context

    # endregion
