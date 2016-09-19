#include <string>
#include <iostream>
#include <cctype>
#include <vector>
#include <stdexcept>
#include <memory>
#include <utility>

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
		std::vector<std::unique_ptr<AST>> children;
		AST(Token _token, std::vector<std::unique_ptr<AST>> _children) : token(_token), children(std::move(_children)) {
		}
		AST(Token _token) : token(_token) {
		}
	public:
		Token getToken() const { return token; }
		AST* getChild(std::size_t i) const {
			return children[i].get();
		}
		std::size_t getNumberChildren() const { return children.size(); }
		virtual ~AST() {
			for (auto& child : children) {
				child.reset();
			}
		}
};
std::ostream &operator<<(std::ostream &os, AST const &m) {
	return os << "AST<token='" << m.getToken() << "', children count=" << m.getNumberChildren() << ">";
}

class BinaryOp : public AST {
	public:
		BinaryOp(std::unique_ptr<AST> left, Token op, std::unique_ptr<AST> right) : AST(op) {
			children.emplace_back(std::move(left));
			children.emplace_back(std::move(right));
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

		std::unique_ptr<AST> factor() {
			if (currentToken.getType() == IntegerToken) {
				auto node = std::unique_ptr<AST>(new Num(currentToken));
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

		std::unique_ptr<AST> term() {
			auto node = factor();

			while (currentToken.getType() == MultiplyToken ||
					currentToken.getType() == DivideToken) {
				Token op = currentToken;
				eat(op.getType());
				auto second_factor = factor();

				node = std::unique_ptr<AST>(new BinaryOp(std::move(node), op, std::move(second_factor)));
			}
			return node;
		}

		std::unique_ptr<AST> expr() {
			auto node = term();

			while (currentToken.getType() == PlusToken ||
					currentToken.getType() == MinusToken) {
				Token op = currentToken;
				eat(op.getType());
				auto second_term = term();

				node = std::unique_ptr<AST>(new BinaryOp(std::move(node), op, std::move(second_term)));
			}
			return node;
		}

	public:
		Parser(Lexer* lexer) : lexer(lexer), currentToken(lexer->get_next_token()) {
		}

		std::unique_ptr<AST> parse() {
			auto tree = expr();
			if (currentToken.getType() != EOFToken)
				throw std::runtime_error("Unexpected characters at end");
			delete lexer;
			return tree;
		}
};

class Interpreter {
	private:
		int visit(AST& node) {
			switch(node.getToken().getType()) {
				case PlusToken:
				case MinusToken:
				case MultiplyToken:
				case DivideToken:
					return visit_BinaryOp(node);
				case IntegerToken:
					return visit_Num(node);
				default:
					throw std::runtime_error("Unknown node type");
			}
		}

		int visit_Num(AST& node) {
			return node.getToken().getValue();
		}

		int visit_BinaryOp(AST& node) {
			if (node.getNumberChildren() != 2)
				throw std::runtime_error("Wrong number of children");

			auto first_value = visit(*node.getChild(0));
			auto second_value = visit(*node.getChild(1));

			switch(node.getToken().getType()) {
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
		int interpret(AST& tree) {
			return visit(tree);
		}
};

int main() {
	std::string input;

	while(true) {
		std::cout << "calc> ";
		getline(std::cin, input);

		auto ast_tree = Parser(new Lexer(input)).parse();
		std::cout << Interpreter().interpret(*ast_tree) << std::endl;
		ast_tree.reset();
	}
}
