class Token
	attr_accessor :value
	attr_accessor :type
	def initialize(type, value)
		raise "Unknown type" unless [
			:INTEGER_CONST,:PLUS,:MINUS,:MULTIPLY,:FLOAT_DIVIDE,:PROGRAM,:SEMI,
			:DOT,:VAR,:ID,:COLON,:REAL_CONST,:INTEGER_DIVIDE,:LPAREN,:RPAREN,
			:BEGIN,:END,:ASSIGN,:COMMA,:COLON,:INTEGER,:REAL,:EOF].include?(type)
		@type = type
		@value = value
	end

	def inspect
		"<Token #{@type}: #{@value}>"
	end

	def to_s
		inspect
	end
end

class Lexer
	def initialize(text)
		@text = text
	end

	def get_next_token
		remove_whitespace_and_comments

		match = /^(\d+\.\d+)/.match @text
		unless match.nil?
			@text = match.post_match
			return Token.new :REAL_CONST, match.to_s.to_f
		end

		match = /^(\d+)/.match @text
		unless match.nil?
			@text = match.post_match
			return Token.new :INTEGER_CONST, match.to_s.to_i
		end

		match = /^(\+)/.match @text
		unless match.nil?
			@text = match.post_match
			return Token.new :PLUS, match.to_s
		end

		match = /^(-)/.match @text
		unless match.nil?
			@text = match.post_match
			return Token.new :MINUS, match.to_s
		end

		match = /^(\*)/.match @text
		unless match.nil?
			@text = match.post_match
			return Token.new :MULTIPLY, match.to_s
		end

		match = /^(\/)/.match(@text)
		unless match.nil?
			@text = match.post_match
			return Token.new(:FLOAT_DIVIDE, match.to_s)
		end

		match = /^(DIV)/.match(@text)
		unless match.nil?
			@text = match.post_match
			return Token.new(:INTEGER_DIVIDE, match.to_s)
		end

		match = /^(;)/.match(@text)
		unless match.nil?
			@text = match.post_match
			return Token.new(:SEMI, match.to_s)
		end

		match = /^(,)/.match(@text)
		unless match.nil?
			@text = match.post_match
			return Token.new(:COMMA, match.to_s)
		end

		match = /^(:=)/.match(@text)
		unless match.nil?
			@text = match.post_match
			return Token.new(:ASSIGN, match.to_s)
		end

		match = /^(:)/.match(@text)
		unless match.nil?
			@text = match.post_match
			return Token.new(:COLON, match.to_s)
		end

		match = /^(\.)/.match(@text)
		unless match.nil?
			@text = match.post_match
			return Token.new(:DOT, match.to_s)
		end

		match = /^(\()/.match(@text)
		unless match.nil?
			@text = match.post_match
			return Token.new(:LPAREN, match.to_s)
		end

		match = /^(\))/.match(@text)
		unless match.nil?
			@text = match.post_match
			return Token.new(:RPAREN, match.to_s)
		end

		match = /^(_?[A-z][A-z0-9]*)/.match(@text)
		unless match.nil?
			@text = match.post_match
			id = match.to_s
			type = :ID
			case id
				when /^VAR$/i then type = :VAR
				when /^PROGRAM$/i then type = :PROGRAM
				when /^BEGIN$/i then type = :BEGIN
				when /^END$/i then type = :END
				when /^INTEGER$/i then type = :INTEGER
				when /^REAL$/i then type = :REAL
			end
			return Token.new(type, id)
		end

		if @text.empty?
			return Token.new(:EOF, nil)
		end

		raise "Unable to parse begining of " + @text
	end

	private

	def remove_whitespace_and_comments
		loop do
			@text = @text.lstrip
			if @text[0] == '{'
				match = /^\{[^}]*\}/.match(@text)
				if match.nil?
					raise "Unmatched comment starting at #{@text}"
				end
				@text = match.post_match
			else
				break
			end
		end
	end
end

class ASTNode
	attr_accessor :token
	def initialize(token)
		@token = token
	end

	def to_s
		inspect
	end

	def inspect
		"<#{self.class.name} token=#{@token.type} value=#{@token.value}>"
	end
end

class NumNode < ASTNode
end

class BinOpNode < ASTNode
	attr_accessor :left, :right
	def initialize(token, left, right)
		super(token)
		@left = left
		@right = right
	end

	def inspect
		"<#{self.class.name} op=#{@token.type} left=#{@left} right=#{@right}>"
	end
end

class VarNode < ASTNode
end

class UnaryOpNode < ASTNode
	attr_accessor :child
	def initialize(token, child)
		super(token)
		@child = child
	end

	def inspect
		"<#{self.class.name} op=#{@token.type} child=#{@child}>"
	end
end

class AssignmentNode < ASTNode
	attr_accessor :lhs, :rhs
	def initialize(lhs, rhs)
		super(nil)
		@lhs = lhs
		@rhs = rhs
	end

	def inspect
		"<#{self.class.name} lhs=#{@lhs} rhs=#{@rhs}>"
	end
end

class TypeNode < ASTNode
	attr_accessor :type
	def initialize(token)
		super(token)
		@type = token.value.upcase
	end
end

class VariableDeclarationNode < ASTNode
	attr_accessor :variable, :type_node
	def initialize(variable_node, type_node)
		super(nil)
		@variable = variable_node
		@type_node = type_node
	end

	def inspect
		"<#{self.class.name} var=#{@variable.value} type=#{@type_node.type}>"
	end
end

class BlockNode < ASTNode
	attr_accessor :declarations, :statements
	def initialize(declarations, statements)
		super(nil)
		@declarations = declarations
		@statements = statements
	end

	def inspect
		"<#{self.class.name} vars=#{@declarations} code=#{@statements}>"
	end
end

class ProgramNode < ASTNode
	attr_accessor :program_name, :block
	def initialize(program_name, block)
		super(nil)
		@program_name = program_name
		@block = block
	end

	def inspect
		"<#{self.class.name} name=#{@program_name.token.value} program=#{@block}>"
	end
end

class Parser
	def initialize(lexer)
		@lexer = lexer
		@current_token = lexer.get_next_token
	end

	def parse
		output = program
		eat :EOF
		output
	end

	private

	def eat(token_type=nil)
		error_msg = "Expected #{token_type} but current token is #{@current_token}"
		if token_type.respond_to?('include?')
			raise error_msg unless token_type.include? @current_token.type
		elsif not token_type.nil?
			raise error_msg unless token_type == @current_token.type
		end

		output = @current_token
		@current_token = @lexer.get_next_token
		output
	end

	def variable
		# variable : ID
		VarNode.new eat :ID
	end

	def factor
		# factor : PLUS factor
		#        | MINUS factor
		#        | INTEGER_CONST
		#        | REAL_CONST
		#        | LPAREN expr RPAREN
		#        | variable
		case @current_token.type
		when :PLUS, :MINUS
			UnaryOpNode.new eat, factor
		when :INTEGER_CONST, :REAL_CONST
			NumNode.new eat
		when :LPAREN
			eat(:LPAREN)
			output = expr
			eat(:RPAREN)
			output
		when :ID
			variable
		end
	end

	def term
		# term : factor ((MULTIPLY | INTEGER_DIVIDE | FLOAT_DIVIDE) factor)*
		output = factor
		while [:MULTIPLY,:INTEGER_DIVIDE,:FLOAT_DIVIDE].include? @current_token.type
			output = BinOpNode.new eat, output, factor
		end
		output
	end

	def expr
		# expr : term ((PLUS | MINUS) term)*
		output = term
		while [:PLUS,:MINUS].include? @current_token.type
			output = BinOpNode.new eat, output, term
		end
		output
	end

	def empty
		# Returns empty array without any parsing
		[]
	end

	def assignment_statement
		# assignment_statement : variable ASSIGN expr
		var = variable
		eat :ASSIGN
		AssignmentNode.new var, expr
	end

	def statement
		# returns array of AssignmentNode objects
		# statement : compound_statement
		#           | assignment_statement
		#           | empty
		case @current_token.type
		when :BEGIN
			return compound_statement
		when :ID
			return [assignment_statement]
		else
			return empty
		end
	end

	def statement_list
		# statement_list : statement
		#                | statement SEMI statement_list
		output = statement
		if @current_token.type == :SEMI
			eat
			output += statement_list
		end
		output
	end

	def compound_statement
		# returns array of AssignmentNode objects
		# compound_statement : BEGIN statement_list END
		eat :BEGIN
		output = statement_list
		eat :END
		output
	end

	def type_spec
		# type_spec : REAL | INTEGER
		TypeNode.new eat([:REAL,:INTEGER])
	end

	def variable_declaration
		# returns Array of VariableDeclarationNode objects
		# variable_declaration : ID (COMMA ID)* COLON type_spec
		ids = [eat(:ID)]
		while @current_token.type == :COMMA
			eat
			ids << eat(:ID)
		end
		eat :COLON
		type = type_spec

		output = []
		for id in ids
			output << VariableDeclarationNode.new(id, type)
		end
		output
	end

	def declarations
		# returns Array of VariableDeclarationNode objects
		# declarations : VAR (variable_declaration SEMI)+
		#              | empty
		if @current_token.type == :VAR
			eat
			output = variable_declaration
			eat :SEMI
			while @current_token.type == :ID
				output += variable_declaration
				eat :SEMI
			end
			return output
		else
			return empty
		end
	end

	def block
		# block : declarations compound_statement
		BlockNode.new declarations, compound_statement
	end

	def program
		# program : PROGRAM variable SEMI block DOT
		eat :PROGRAM
		name = variable
		eat :SEMI
		output = ProgramNode.new name, block
		eat :DOT
		return output
	end
end

class NodeVisiter
	def visit(node)
		node_name = node.class.name
		self.public_send "visit_" + node_name, node
	end
end

class Interpreter < NodeVisiter
	def initialize(ast_tree)
		@ast_tree = ast_tree
		@GLOBAL = Hash.new
	end

	def interpret
		visit @ast_tree
	end

	def visit_ProgramNode(node)
		puts "---------Program: #{node.program_name.token.value}---------"
		visit node.block
	end

	def visit_BlockNode(node)
		puts "declarations:"
		for decl in node.declarations
			visit decl
		end

		puts "Evaluating statements:"
		for statement in node.statements
			visit statement
		end
	end

	def visit_VariableDeclarationNode(node)
		puts "\t#{node.variable.value} : #{node.type_node.type}"
	end

	def visit_AssignmentNode(node)
		variable_name = node.lhs.token.value
		rhs = visit node.rhs
		puts "...Assigning #{variable_name} to #{rhs}"
		@GLOBAL[node.lhs.token.value] = rhs
	end

	def visit_NumNode(node)
		node.token.value
	end

	def visit_VarNode(node)
		variable_name = node.token.value
		unless @GLOBAL.include?(variable_name)
			raise "Variable #{variable_name} not found"
		end
		@GLOBAL[variable_name]
	end

	def visit_BinOpNode(node)
		case node.token.type
		when :PLUS then visit(node.left) + visit(node.right)
		when :MINUS then visit(node.left) - visit(node.right)
		when :MULTIPLY then visit(node.left) * visit(node.right)
		when :FLOAT_DIVIDE then visit(node.left).to_f / visit(node.right)
		when :INTEGER_DIVIDE then (visit(node.left) / visit(node.right)).to_i
		else raise "Unknown Binary Operations: #{node}"
		end
	end

	def visit_UnaryOpNode(node)
		case node.token.type
		when :PLUS then visit(node.child)
		when :MINUS then -visit(node.child)
		else raise "Unknown Unary Operations: #{node}"
		end
	end
end

filename = ARGV.first
txt = open(filename).read
Interpreter.new(Parser.new(Lexer.new(txt)).parse()).interpret
