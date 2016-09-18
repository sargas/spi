#include <string>
#include <iostream>
#include <cctype>
#include <vector>
#include <stdexcept>

enum token_type_t {IntegerToken, PlusToken, MinusToken, MultiplyToken, DivideToken, LParenToken, RParenToken, EOFToken};

class Token {
	private:
		token_type_t type;
		int value;

	public:
		Token(token_type_t type, int value = 0) : type(type), value(value) {
		}
		token_type_t getType() const { return type; }
		int getValue() const { return value; }
};
std::ostream &operator<<(std::ostream &os, Token const &m) {
	return os << "Token<" << m.getType() << ", " << m.getValue() << ">";
}

class Lexer {
	private:
		std::string text;
		std::size_t position;
		char currentChar;

		void advance() {
			position++;
			if (position > text.length() - 1) {
				currentChar = 0;
			} else {
				currentChar = text[position];
			}
		}
		void skip_whitespace() {
			while (currentChar != 0 && isspace(currentChar))
				advance();
		}
		int integer() {
			const std::size_t initialPosition = position;
			std::size_t length = 0;

			while (currentChar != 0 && isdigit(currentChar)) {
				length++;
				advance();
			}

			return std::stoi(text.substr(initialPosition, length), 0, 10);
		}
	public:
		Lexer(std::string text) : text(text), position(0) {
			currentChar = this->text[0];
		}

		Token get_next_token() {
			while (currentChar != 0) {
				if (isspace(currentChar)) {
					skip_whitespace();
					continue;
				}
				if (isdigit(currentChar)) {
					return Token(IntegerToken, integer());
				}

				switch (currentChar) {
					case '+':
						advance();
						return Token(PlusToken);
					case '-':
						advance();
						return Token(MinusToken);
					case '*':
						advance();
						return Token(MultiplyToken);
					case '/':
						advance();
						return Token(DivideToken);
					case '(':
						advance();
						return Token(LParenToken);
					case ')':
						advance();
						return Token(RParenToken);
					default:
						throw std::runtime_error("Unknown token");
				}
			}
			return Token(EOFToken);
		}
};

class AST {
	protected:
		Token token;
		std::vector<AST*> children;
		AST(Token _token, std::vector<AST*> _children) : token(_token), children(_children) {
		}
		AST(Token _token) : token(_token) {
		}
	public:
		Token getToken() const { return token; }
		std::vector<AST*> getChildren() const {
			return children;
		}
		virtual ~AST() {
			for (auto child : children) {
				delete child;
			}
		}
};
std::ostream &operator<<(std::ostream &os, AST const &m) {
	return os << "AST<token='" << m.getToken() << "', children count=" << m.getChildren().size() << ">";
}

class BinaryOp : public AST {
	public:
		BinaryOp(AST* left, Token op, AST* right) : AST(op, {left, right}) {
		}
};

class Num : public AST {
	public:
		Num(Token token) : AST(token) {
		}
};

class Parser {
	/*
	 * expr : term ((+|-) term)*
	 * term : factor ((*|/) term)*
	 * factor : INTEGER | LPAREN expr RPAREN
	 */
	private:
		Lexer* lexer;
		Token currentToken;

		void eat(token_type_t type) {
			if (currentToken.getType() == type) {
				currentToken = lexer->get_next_token();
			} else
				throw std::runtime_error("Syntax error");
		}

		AST* factor() {
			if (currentToken.getType() == IntegerToken) {
				auto node = new Num(currentToken);
				eat(IntegerToken);
				return node;
			} else if (currentToken.getType() == LParenToken) {
				eat(LParenToken);
				auto node = expr();
				eat(RParenToken);
				return node;
			} else {
				throw std::runtime_error("Expected integer or left parenthesis, got neither");
			}
		}

		AST* term() {
			AST* node = factor();

			while (currentToken.getType() == MultiplyToken ||
					currentToken.getType() == DivideToken) {
				Token op = currentToken;
				eat(op.getType());

				node = new BinaryOp(node, op, factor());
			}
			return node;
		}

		AST* expr() {
			AST* node = term();

			while (currentToken.getType() == PlusToken ||
					currentToken.getType() == MinusToken) {
				Token op = currentToken;
				eat(op.getType());

				node = new BinaryOp(node, op, term());
			}
			return node;
		}

	public:
		Parser(Lexer* lexer) : lexer(lexer), currentToken(lexer->get_next_token()) {
		}

		AST* parse() {
			auto tree = expr();
			if (currentToken.getType() != EOFToken)
				throw std::runtime_error("Unexpected characters at end");
			delete lexer;
			return tree;
		}
};

class Interpreter {
	private:
		int visit(AST* node) {
			switch(node->getToken().getType()) {
				case PlusToken:
				case MinusToken:
				case MultiplyToken:
				case DivideToken:
					return visit_BinaryOp(node);
				case IntegerToken:
					return node->getToken().getValue();
				default:
					throw std::runtime_error("Unknown node type");
			}
		}

		int visit_BinaryOp(AST* node) {
			if (node->getChildren().size() != 2)
				throw std::runtime_error("Wrong number of children");

			auto first_value = visit(node->getChildren()[0]);
			auto second_value = visit(node->getChildren()[1]);

			switch(node->getToken().getType()) {
				case PlusToken:
					return first_value + second_value;
				case MinusToken:
					return first_value - second_value;
				case MultiplyToken:
					return first_value * second_value;
				case DivideToken:
					return first_value / second_value;
				default:
					throw std::runtime_error("Unknown node type");
			}
		}

	public:
		int interpret(AST* tree) {
			return visit(tree);
		}
};

int main() {
	std::string input;

	while(true) {
		std::cout << "calc> ";
		getline(std::cin, input);

		auto ast_tree = Parser(new Lexer(input)).parse();
		std::cout << Interpreter().interpret(ast_tree) << std::endl;
		delete ast_tree;
	}
}
