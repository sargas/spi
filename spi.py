from enum import Enum
import re
import sys


class TokenTypes(Enum):
    INTEGER = r'\d+'
    PLUS = r'\+'
    MINUS = r'\-'
    MULTIPLY = r'\*'
    DIVIDE = r'/'
    LPAREN = r'\('
    RPAREN = r'\)'
    DOT = r'\.'
    ASSIGN = r':='
    ID = r'[A-za-z]+'
    SEMI = r';'
    BEGIN = r'BEGIN'
    END = r'END'
    EOF = r'$^'

    def __init__(self, regex):
        self.regex = re.compile(regex)

    def __repr__(self):
        return '<TokenType.{}>'.format(self.name)

RESERVED_KEYWORDS = ['BEGIN', 'END']


class Token:
    def __init__(self, token_type, value):
        assert token_type in TokenTypes or token_type is None
        self.token_type = token_type
        self.value = value

    def __str__(self):
        return 'Token({0.token_type}, {0.value!r})'.format(self)

    def __repr__(self):
        return self.__str__()


class Lexer:
    def __init__(self, text):
        self.text = self._current_text = text

    def get_next_token(self):
        text = self._current_text.lstrip()

        for token_type in TokenTypes:
            match = token_type.regex.match(text)
            if match:
                if token_type == TokenTypes.ID and \
                   match.group() in RESERVED_KEYWORDS:
                    continue

                self._current_text = text[match.end():]
                return Token(token_type, match.group())
        if len(text) > 0:
            raise Exception("Couldn't tokenize {}".format(text))
        return Token(TokenTypes.EOF, None)


class ASTNode:
    def __init__(self, token):
        self.token = token


class BinaryOpNode(ASTNode):
    def __init__(self, left, op, right):
        super().__init__(op)
        self.left = left
        self.right = right

    def __repr__(self):
        return "<BinaryOp({0.left},{0.token.value},{0.right})>".format(self)


class UnaryOpNode(ASTNode):
    def __init__(self, op, child):
        super().__init__(op)
        self.child = child

    def __repr__(self):
        return "<UnaryOp({0.token.value},{0.child})>".format(self)


class NumNode(ASTNode):
    def __init__(self, token):
        super().__init__(token)
        self.value = int(token.value)

    def __repr__(self):
        return "<Var({0.token.value})>".format(self)


class CompoundNode(ASTNode):
    def __init__(self, children):
        super().__init__(None)
        self.children = children

    def __repr__(self):
        return "<Compound({0.children!s})>".format(self)


class AssignNode(ASTNode):
    def __init__(self, lvalue, op, rvalue):
        super().__init__(op)
        self.lvalue = lvalue
        self.rvalue = rvalue

    def __repr__(self):
        return "<Assign({0.lvalue},{0.token.value},{0.rvalue})>".format(self)


class VarNode(ASTNode):
    def __init__(self, token):
        super().__init__(token)
        self.token = token

    def __repr__(self):
        return "<Var({0.token.value})>".format(self)


class NoOpNode(ASTNode):
    def __init__(self):
        super().__init__(None)

    def __repr__(self):
        return '<NoOp>'


class Parser:
    '''
    program : compound_statement DOT

    compound_statement : BEGIN statement_list END

    statement_list : statement | statement SEMI statement_list

    statement : compound_statement | assignment_statement | empty

    assignment_statement : variable ASSIGN expr

    empty :

    expr : term ((PLUS|MINUS) term)*

    term : factor ((MULT|DIVIDE) factor)*

    factor : PLUS factor | MINUS factor | INTERGER
            | LPAREN expr RPAREN | variable

    variable : ID
    '''
    def __init__(self, lexer):
        self._lexer = lexer

    def _eat(self, previous=None):
        output = self._current_token
        if previous is not None and \
           output.token_type != previous and \
           (not isinstance(previous, list) or output.token_type not in previous):
            raise Exception("Expected to see {0!r}, found {1!r} instead".
                            format(previous, output))
        self._current_token = self._lexer.get_next_token()
        return output

    def _program(self):
        node = self._compound_statement()
        self._eat(TokenTypes.DOT)
        return node

    def _compound_statement(self):
        self._eat(TokenTypes.BEGIN)
        node = self._statement_list()
        self._eat(TokenTypes.END)
        return node

    def _statement_list(self):
        statements = [self._statement()]
        while self._current_token.token_type == TokenTypes.SEMI:
            self._eat(TokenTypes.SEMI)
            statements.append(self._statement())
        return CompoundNode(statements)

    def _statement(self):
        if self._current_token.token_type == TokenTypes.BEGIN:
            return self._compound_statement()
        elif self._current_token.token_type == TokenTypes.ID:
            return self._assignment_statement()
        else:
            return self._empty()

    def _assignment_statement(self):
        lvalue = self._variable()
        token = self._current_token
        self._eat(TokenTypes.ASSIGN)
        rvalue = self._expr()
        return AssignNode(lvalue, token, rvalue)

    def _variable(self):
        return VarNode(self._eat(TokenTypes.ID))

    def _empty(self):
        return NoOpNode()

    def _expr(self):
        result = self._term()
        while self._current_token.token_type in [TokenTypes.PLUS,
                                                 TokenTypes.MINUS]:
            op = self._eat()
            result = BinaryOpNode(result, op, self._term())
        return result

    def _term(self):
        result = self._factor()
        while self._current_token.token_type in [TokenTypes.MULTIPLY,
                                                 TokenTypes.DIVIDE]:
            op = self._eat()
            result = BinaryOpNode(result, op, self._factor())
        return result

    def _factor(self):
        if self._current_token.token_type in [TokenTypes.PLUS,
                                              TokenTypes.MINUS]:
            op = self._eat()
            return UnaryOpNode(op, self._factor())
        elif self._current_token.token_type == TokenTypes.INTEGER:
            return NumNode(self._eat())
        elif self._current_token.token_type == TokenTypes.LPAREN:
            self._eat(TokenTypes.LPAREN)
            node = self._expr()
            self._eat(TokenTypes.RPAREN)
            return node
        else:
            return self._variable()

    def parse(self):
        self._current_token = self._lexer.get_next_token()
        node = self._program()
        self._eat(TokenTypes.EOF)
        return node


class NodeVisiter:
    def visit(self, node):
        method_name = 'visit_' + type(node).__name__
        visitor = getattr(self, method_name, self.generic_visit)
        return visitor(node)

    def generic_visit(self, node):
        raise Exception('No visit_{} method'.format(type(node).__name__))


class Interpreter(NodeVisiter):
    def __init__(self):
        self.GLOBAL_SCOPE = {}

    def visit_NumNode(self, node):
        return node.value

    def visit_UnaryOpNode(self, node):
        inner_value = self.visit(node.child)
        if node.token.token_type == TokenTypes.MINUS:
            return -inner_value
        else:
            return inner_value

    def visit_BinaryOpNode(self, node):
        first_value = self.visit(node.left)
        second_value = self.visit(node.right)
        return {TokenTypes.PLUS: lambda x, y: x+y,
                TokenTypes.MINUS: lambda x, y: x-y,
                TokenTypes.MULTIPLY: lambda x, y: x*y,
                TokenTypes.DIVIDE: lambda x, y: x//y,
                }[node.token.token_type](first_value, second_value)

    def visit_CompoundNode(self, node):
        for child in node.children:
            self.visit(child)

    def visit_AssignNode(self, node):
        var_name = node.lvalue.token.value
        self.GLOBAL_SCOPE[var_name] = self.visit(node.rvalue)

    def visit_VarNode(self, node):
        var_name = node.token.value
        if var_name not in self.GLOBAL_SCOPE:
            raise Exception("{} used before being assigned".format(var_name))
        else:
            return self.GLOBAL_SCOPE[var_name]

    def visit_NoOpNode(self, node):
        pass


def interpret_and_print(text):
    ast = Parser(Lexer(text)).parse()
    interpreter = Interpreter()
    interpreter.visit(ast)
    print(interpreter.GLOBAL_SCOPE)


def main():
    if len(sys.argv) > 1:
        with open(sys.argv[1], 'r') as f:
            interpret_and_print(f.read())
    else:
        while True:
            try:
                text = input("spi> ")
            except EOFError:
                break
            if not text:
                continue
            interpret_and_print(text)

if __name__ == '__main__':
    main()
