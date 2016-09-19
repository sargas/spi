# vim: filetype=perl6
use v6;

enum TokenType <INTEGER PLUS MINUS MULTIPLY DIVIDE LPAREN RPAREN EOF>;

class Token {
	has TokenType $.type;
	has $.value = '';
}

class Lexer {
	has Str $.text;

	method !skip_whitespace {
		$!text .= trim-leading;
	}

	method !integer returns Str {
		if $!text ~~ /^\d+/ {
			$!text = $/.postmatch;
			~$/;
		}
	}

	method !advance_one returns Str {
		$!text .= substr(1);

	}

	method get_next_token returns Token {
		self!skip_whitespace();
		return Token.new(type=>EOF) unless $!text;

		given $!text[0] {
			when /^\d+/ { return Token.new(type=>INTEGER, value=>self!integer());}
			when /^\+/ { return Token.new(type=>PLUS, value=>self!advance_one());}
			when /^\-/ { return Token.new(type=>MINUS, value=>self!advance_one());}
			when /^\*/ { return Token.new(type=>MULTIPLY, value=>self!advance_one());}
			when /^\// { return Token.new(type=>DIVIDE, value=>self!advance_one());}
			when /^\(/ { return Token.new(type=>LPAREN, value=>self!advance_one());}
			when /^\)/ { return Token.new(type=>RPAREN, value=>self!advance_one());}
			default { die "Unexpected character at beginning of '" + $!text + "'" }
		}
	}
}

class ASTNode {
	has Token $.token;
}
class NumNode is ASTNode {
	has int $.value;
}
class BinaryOpNode is ASTNode {
	has ASTNode $.left;
	has ASTNode $.right;
}
class UnaryOpNode is ASTNode {
	has ASTNode $.child;
}

class Parser {
	# expr : term ((+|-) term)*
	# term : factor ((*|/) factor)*
	# factor : (+|-) factor | LPAREN expr RPAREN | integer
	#
	has Lexer $.lexer;
	has Token $!currentToken;

	method !eat (TokenType $previous?) {
		die "Syntax Error" if $previous and $!currentToken.type != $previous;
		$!currentToken = $!lexer.get_next_token();
	}

	method !factor returns ASTNode {
		given $!currentToken.type {
			when INTEGER {
				my $node = NumNode.new(token=>$!currentToken, value=>+$!currentToken.value);
				self!eat(INTEGER);
				return $node;
			}
			when LPAREN {
				self!eat(LPAREN);
				my $node =self!expr;
				self!eat(RPAREN);
				return $node;
			}
			when PLUS ?| MINUS {
				my $token = $!currentToken;
				self!eat();
				return UnaryOpNode.new(token=>$token, child=>self!factor);
			}
			default { die "Syntax Error"; }
		}
	}

	method !term returns ASTNode {
		my $result = self!factor;
		while $!currentToken.type ∈ [MULTIPLY, DIVIDE] {
			my $op = $!currentToken;
			self!eat();
			$result = BinaryOpNode.new(token=>$op, left=>$result, right=>self!factor);
		}
		return $result;
	}

	method !expr returns ASTNode {
		my $result = self!term;
		while $!currentToken.type ∈ [PLUS, MINUS] {
			my $op = $!currentToken;
			self!eat();
			$result = BinaryOpNode.new(token=>$op, left=>$result, right=>self!term);
		}
		return $result;
	}

	method parse returns ASTNode {
		$!currentToken = $!lexer.get_next_token();
		return self!expr;
	}
}

class NodeVisiter {
	has ASTNode $.tree;

	method visit(ASTNode $tree) {
		given $tree.WHAT {
			when NumNode.WHAT { return self.visit_Num($tree) }
			when BinaryOpNode.WHAT { return self.visit_BinaryOp($tree) }
			when UnaryOpNode.WHAT { return self.visit_UnaryOp($tree) }
		}
	}

	method visit_Num($node) { ... }

	method interpret {
		self.visit($.tree);
	}
}

class Calculator is NodeVisiter {
	method visit_Num(NumNode $node) returns int {
		return +$node.value;
	}
	method visit_BinaryOp(BinaryOpNode $node) returns int {
		my $left = +self.visit($node.left);
		my $right = +self.visit($node.right);
		given $node.token.type {
			when PLUS {$left + $right}
			when MINUS {$left - $right}
			when MULTIPLY {$left * $right}
			when DIVIDE {Int($left / $right)}
		}
	}
	method visit_UnaryOp(UnaryOpNode $node) returns int {
		my $child = +self.visit($node.child);
		given $node.token.type {
			when PLUS {$child}
			when MINUS {0 - $child}
		}
	}
}
class RPNPrinter is NodeVisiter {
	method visit_Num(NumNode $node) returns Str {
		return ~$node.value;
	}
	method visit_BinaryOp(BinaryOpNode $node) returns Str {
		my $left = self.visit($node.left);
		my $right = self.visit($node.right);
		given $node.token.type {
			when PLUS {"$left $right +"}
			when MINUS {"$left $right -"}
			when MULTIPLY {"$left $right *"}
			when DIVIDE {"$left $right /"}
		}
	}
	method visit_UnaryOp(UnaryOpNode $node) returns Str {
		my $child = self.visit($node.child);
		given $node.token.type {
			when PLUS {"+$child"}
			when MINUS {"-$child"}
		}
	}
}
class LispPrinter is NodeVisiter {
	method visit_Num(NumNode $node) returns Str {
		return ~$node.value;
	}
	method visit_BinaryOp(BinaryOpNode $node) returns Str {
		my $left = self.visit($node.left);
		my $right = self.visit($node.right);
		given $node.token.type {
			when PLUS {"(+ $left $right)"}
			when MINUS {"(- $left $right)"}
			when MULTIPLY {"(* $left $right)"}
			when DIVIDE {"(/ $left $right)"}
		}
	}
	method visit_UnaryOp(UnaryOpNode $node) returns Str {
		my $child = self.visit($node.child);
		given $node.token.type {
			when PLUS {"+$child"}
			when MINUS {"-$child"}
		}
	}
}

loop {
	my $text = prompt "calc> ";
	next unless $text;
	my $lexer = Lexer.new(text=>$text);
	my $tree = Parser.new(lexer=>$lexer).parse;
	#say 'RPN: ', RPNPrinter.new(tree=>$tree).interpret;
	say 'Lisp: ', LispPrinter.new(tree=>$tree).interpret;
	say 'Answer: ', Calculator.new(tree=>$tree).interpret;
	say '';
}
