from enum import Enum
import re

# Token types
class TokenType(Enum):
    INTEGER = (r'\d+', lambda x: int(x), None)
    PLUS = (r'\+', lambda x: x, lambda x, y: x + y)
    MINUS = (r'-', lambda x: x, lambda x, y: x - y)
    MULTIPLY = (r'\*', lambda x: x, lambda x, y: x * y)
    DIVIDE = (r'/', lambda x: x, lambda x, y: x // y)
    LPAREN = (r'\(', lambda x: x, None)
    RPAREN = (r'\)', lambda x: x, None)

    EOF = ('a^', lambda x: None, None)

    def __init__(self, regex, converter, op):
        self.regex = re.compile(regex)
        self.converter = converter
        self.op = op

    @classmethod
    def is_valid(cls, value):
        if value is None:
            return True
        for type in cls:
            if type.regex.match(str(value)):
                return True
        return False

    def extract_info(self, value):
        match = self.regex.match(value)
        return (self.converter(match.group(0)),
                value[match.end():])

    def __repr__(self):
        return '<TokenType.{}>'.format(self.name)

OPS_1 = [TokenType.MULTIPLY, TokenType.DIVIDE]
OPS_2 = [TokenType.PLUS, TokenType.MINUS]

class Token:
    def __init__(self, type, value):
        assert type in TokenType
        assert TokenType.is_valid(value)

        self.type = type
        self.value = value

    def __str__(self):
        return 'Token({0.type}, {0.value!r})'.format(self)

    def __repr__(self):
        return self.__str__()


class Interpreter:
    """
    Rules:
        second : first ( (PLUS|MINUS) first )*
        first : integer ( (MULT|DIV) integer )*
    """
    def __init__(self, text):
        self._text = text
        self._current_token = None

    def _get_next_token(self):
        text = self._text.lstrip()

        if len(text) == 0:
            return Token(TokenType.EOF, None)

        for token_type in TokenType:
            if token_type.regex.match(text):
                value, new_text = token_type.extract_info(text)
                token = Token(token_type, value)
                self._text = new_text
                return token

        raise Exception("Didn't recognize first token for '{}'".format(
            text))

    def _parse_int_or_parens(self):
        if self._current_token.type == TokenType.LPAREN:
            self._eat([TokenType.LPAREN])
            result = self._parse_second_layer()
            self._eat([TokenType.RPAREN])
        elif self._current_token.type == TokenType.INTEGER:
            result = self._current_token.value
            self._eat([TokenType.INTEGER])
        else:
            raise Exception("Expected int or parenthesis for '{}'".format(
                self._text))
        return result

    def _parse_first_layer(self):
        result = self._parse_int_or_parens()

        while self._current_token.type in OPS_1:
            op = self._current_token
            self._eat(OPS_1)

            right_int = self._parse_int_or_parens()
            result = op.type.op(result, right_int)

        return result

    def _parse_second_layer(self):
        result = self._parse_first_layer()

        while self._current_token.type in OPS_2:
            op = self._current_token
            self._eat(OPS_2)

            right_factor = self._parse_first_layer()
            result = op.type.op(result, right_factor)

        return result

    def _eat(self, token_types):
        for token_type in token_types:
            if self._current_token.type == token_type:
                self._current_token = self._get_next_token()
                return

        raise Exception('Expected one of {}, but current token is {}'.format(
                token_types, self._current_token))

    def expr(self):
        self._current_token = self._get_next_token()
        result = self._parse_second_layer()
        self._eat([TokenType.EOF])
        return result


def main():
    while True:
        try:
            text = input('calc> ')
        except EOFError:
            break
        if not text:
            continue
        interpreter = Interpreter(text)
        result = interpreter.expr()
        print(result)

TEST_CASES = [
    ('2', 2), ('2 * 3', 6), ('  2+3+4', 9), ('2*3*4  ', 24), ('1+2*  3', 7),
    ('42* ( 2 +1)', 126), ('(2*3+36) / (2*(1+1))', 10),
    ('7 + 3 * (10 / (12 / (3 + 1) - 1))', 22)]
if __name__ == '__main__':
    for x, y in TEST_CASES:
        assert Interpreter(x).expr() == y
    print("Tests Passed")
    main()
