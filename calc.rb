class Token
	attr_accessor :value
	def initialize(type, value)
		raise "Unknown type" unless [:INTEGER,:PLUS,:MINUS,:MULTIPLY,:DIVIDE,:EOF].include?(type)
		@type = type
		@value = value
	end
end

class Interpreter
	def initialize(text)
		@text = text
	end

	def remove_whitespace
		@text = @text.lstrip
	end

	def parse_int
		remove_whitespace
		match = /^(\d+)/.match(@text)
		unless match.nil?
			@text = match.post_match
			@current_token = Token.new(:INTEGER, match.to_s.to_i)
		end
	end

	def parse_plus
		remove_whitespace
		match = /^\+/.match(@text)
		unless match.nil?
			@text = match.post_match
			@current_token = Token.new(:PLUS, -> (x, y) {x + y})
		end
	end

	def parse_minus
		remove_whitespace
		match = /^-/.match(@text)
		unless match.nil?
			@text = match.post_match
			@current_token = Token.new(:MINUS, -> (x, y) {x - y})
		end
	end

	def parse_mult
		remove_whitespace
		match = /^\*/.match(@text)
		unless match.nil?
			@text = match.post_match
			@current_token = Token.new(:MULTIPLY, -> (x, y) {x * y})
		end
	end

	def parse_div
		remove_whitespace
		match = /^\//.match(@text)
		unless match.nil?
			@text = match.post_match
			@current_token = Token.new(:DIVIDE, -> (x, y) {x / y})
		end
	end

	def parse_eof
		remove_whitespace
		@text.empty?
	end

	def parse_first_level
		raise "Expected int, got #{@text}" unless parse_int
		result = @current_token.value

		while parse_mult or parse_div
			op = @current_token.value
			raise "Expected int, got #{@text}" unless parse_int
			result = op.call(result, @current_token.value)
		end
		return result
	end

	def parse_second_level
		result = parse_first_level

		while parse_plus or parse_minus
			op = @current_token.value
			right_side = parse_first_level
			result = op.call(result, right_side)
		end
		return result
	end

	def expr
		result = parse_second_level
		raise "Unexpected characters: #{@text}" unless parse_eof
		result
	end
end

[['2', 2], ['2 * 3', 6], ['  2+3+4', 9], ['2*3*4  ', 24], ['1+2*  3', 7]].each { |x, y| raise "Test for '#{x}}' and '#{y}' failed" if Interpreter.new(x).expr != y}
puts "Tests Passed"

while TRUE
	print "calc> "
	expr = gets.strip
	puts Interpreter.new(expr).expr
end
