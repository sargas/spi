from enum import Enum
import re

# Token types
class TokenType(Enum):
    INTEGER = (r'\d+', lambda x: int(x), None)
    PLUS = (r'\+', lambda x: x, lambda x, y: x + y)
    MINUS = (r'-', lambda x: x, lambda x, y: x - y)
    MULTIPLY = (r'\*', lambda x: x, lambda x, y: x * y)
    DIVIDE = (r'/', lambda x: x, lambda x, y: x // y)

    COMPOUND_EXPRESSION = (r'\d+', lambda x: x, None)
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

    def _parse_first_layer(self):
        result = self._current_token.value
        self._eat([TokenType.INTEGER])

        while self._current_token.type in OPS_1:
            op = self._current_token
            self._eat(OPS_1)

            right_int = self._current_token.value
            self._eat([TokenType.INTEGER])
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
        return self._parse_second_layer()


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

if __name__ == '__main__':
    main()
