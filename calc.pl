# vim: filetype=perl6
use v6;

enum TokenType <PLUS MINUS INTEGER EOF>;

class Token {
	has $.value;
	has TokenType $.type;
}

class Interpreter {
	has Str $.text;

	method parse_integer {
		self.trim_whitespace;
		if $!text ~~ m/^\d+/ {
			$!text = $/.postmatch;
			Token.new(value => +$/, type => INTEGER);
		} else {
			die "Expected an integer, instead seeing " ~ $!text;
		}
	}

	method parse_plus {
		self.trim_whitespace;
		if $!text ~~ m/^\+/ {
			$!text = $/.postmatch;
			Token.new(value => ~$/, type => PLUS);
		}
	}

	method parse_minus {
		self.trim_whitespace;
		if $!text ~~ m/^\-/ {
			$!text = $/.postmatch;
			Token.new(value => ~$/, type => MINUS);
		}
	}

	method trim_whitespace {
		$!text .= trim-leading;
	}

	method expr {
		my $result = self.parse_integer.value;
		while $!text {
			if self.parse_plus {
				$result += self.parse_integer.value;
			} elsif self.parse_minus {
				$result -= self.parse_integer.value;
			} else {
				die "Expected plus or minus, instead seeing " ~ $!text;
			}
		}
		return $result;
	}
}

loop {
	my $text = prompt "calc> ";
	next unless $text;
	say Interpreter.new(text => $text).expr;
}
