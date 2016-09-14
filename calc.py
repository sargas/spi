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

OPS = [TokenType.PLUS, TokenType.MINUS, TokenType.MULTIPLY, TokenType.DIVIDE]

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

    def _eat(self, token_types):
        for token_type in token_types:
            if self._current_token.type == token_type:
                self._current_token = self._get_next_token()
                return

        raise Exception('Expected one of {}, but current token is {}'.format(
                token_types, self._current_token))

    def expr(self):
        ''' Expect self._text to be INTEGER OP INTEGER'''

        self._current_token = self._get_next_token()

        # eat the first token (should be an int)
        left = self._current_token
        self._eat([TokenType.INTEGER])

        while self._current_token.type != TokenType.EOF:
            # now should have a OP
            op = self._current_token
            self._eat(OPS)

            # final integer
            right = self._current_token
            self._eat([TokenType.INTEGER])

            result = op.type.op(left.value, right.value)
            left = Token(TokenType.COMPOUND_EXPRESSION, result)

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

if __name__ == '__main__':
    main()
