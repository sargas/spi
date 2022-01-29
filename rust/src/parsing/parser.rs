use crate::lexing::token::{Keyword, Token};
use crate::parsing::ast::Ast::{Block, Program};
use crate::parsing::ast::{Ast, TypeSpec, Variable};
use anyhow::bail;

pub struct Parser<I: Iterator<Item = anyhow::Result<Token>>> {
    current_token: Token,
    tokens: I,
}

macro_rules! eat {
    ( $self:ident, $token:pat ) => {
        match &$self.current_token {
            $token => {
                $self.advance()?;
            }
            t => bail!("Expected {:?}, found {:?}", stringify!($token), t),
        };
    };
}

impl<I: Iterator<Item = anyhow::Result<Token>>> Parser<I> {
    pub fn new(tokens: I) -> Parser<I> {
        Parser {
            current_token: Token::Eof,
            tokens,
        }
    }

    fn advance(&mut self) -> anyhow::Result<()> {
        self.current_token = self
            .tokens
            .next()
            .unwrap_or(Ok(Token::Eof))
            .unwrap_or(Token::Eof);
        Ok(())
    }

    /// factor : (PLUS | MINUS) factor | INTEGER_CONST | REAL_CONST | LPAREN expr RPAREN | variable
    fn factor(&mut self) -> anyhow::Result<Ast> {
        match self.current_token {
            Token::Plus => {
                self.advance()?;
                Ok(Ast::PositiveUnary(Box::from(self.factor()?)))
            }
            Token::Minus => {
                self.advance()?;
                Ok(Ast::NegativeUnary(Box::from(self.factor()?)))
            }
            Token::IntegerConstant(i) => {
                self.advance()?;
                Ok(Ast::IntegerConstant(i))
            }
            Token::RealConstant(r) => {
                self.advance()?;
                Ok(Ast::RealConstant(r))
            }
            Token::ParenthesisStart => {
                self.advance()?;
                let nested_result = self.expr();
                eat!(self, Token::ParenthesisEnd);
                nested_result
            }
            Token::Identifier(_) => self.variable(),
            _ => bail!(
                "Expected integer, parenthesis, or variable instead of {:?}",
                self.current_token
            ),
        }
    }

    /// term : factor ((MUL | INTEGER_DIV | REAL_DIV) factor)*
    fn term(&mut self) -> anyhow::Result<Ast> {
        let mut result = self.factor()?;

        loop {
            match self.current_token {
                Token::Multiply => {
                    self.advance()?;
                    result = Ast::Multiply(Box::from(result), Box::from(self.factor()?));
                }
                Token::Keyword(Keyword::IntegerDiv) => {
                    self.advance()?;
                    result = Ast::IntegerDivide(Box::from(result), Box::from(self.factor()?));
                }
                Token::RealDivision => {
                    self.advance()?;
                    result = Ast::RealDivide(Box::from(result), Box::from(self.factor()?));
                }
                _ => {
                    break;
                }
            }
        }
        Ok(result)
    }

    fn expr(&mut self) -> anyhow::Result<Ast> {
        let mut result = self.term()?;

        loop {
            match self.current_token {
                Token::Plus => {
                    self.advance()?;
                    result = Ast::Add(Box::from(result), Box::from(self.term()?));
                }
                Token::Minus => {
                    self.advance()?;
                    result = Ast::Subtract(Box::from(result), Box::from(self.term()?));
                }
                _ => {
                    break;
                }
            }
        }

        Ok(result)
    }

    /// An empty production
    fn empty(&mut self) -> anyhow::Result<Ast> {
        Ok(Ast::NoOp)
    }

    /// variable : ID
    fn variable(&mut self) -> anyhow::Result<Ast> {
        if let Token::Identifier(variable_name) = &self.current_token {
            let name = variable_name.clone();
            self.advance()?;
            Ok(Ast::Variable(Variable { name }))
        } else {
            bail!("Expected a variable, found {:?}", self.current_token)
        }
    }

    /// assignment_statement : variable ASSIGN expr
    fn assignment_statement(&mut self) -> anyhow::Result<Ast> {
        let var_node = self.variable()?;

        eat!(self, Token::Assign);
        let variable = var_node.variable()?.clone();
        Ok(Ast::Assign(variable, Box::from(self.expr()?)))
    }

    /// statement : compound_statement
    ///               | assignment_statement
    ///               | empty
    fn statement(&mut self) -> anyhow::Result<Ast> {
        match &self.current_token {
            Token::Keyword(Keyword::Begin) => self.compound_statement(),
            Token::Identifier(_) => self.assignment_statement(),
            _ => self.empty(),
        }
    }

    /// statement_list : statement
    ///                    | statement SEMI statement_list
    fn statement_list(&mut self) -> anyhow::Result<Vec<Ast>> {
        let mut statements = vec![self.statement()?];
        while let &Token::Semi = &self.current_token {
            self.advance()?;
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    /// compound_statement: BEGIN statement_list END
    fn compound_statement(&mut self) -> anyhow::Result<Ast> {
        eat!(self, Token::Keyword(Keyword::Begin));
        let statements = self.statement_list()?;
        eat!(self, Token::Keyword(Keyword::End));

        Ok(Ast::Compound { statements })
    }

    /// type_spec : INTEGER | REAL
    fn type_spec(&mut self) -> anyhow::Result<TypeSpec> {
        let output = Ok(match &self.current_token {
            Token::Keyword(Keyword::Integer) => TypeSpec::Integer,
            Token::Keyword(Keyword::Real) => TypeSpec::Real,
            token => bail!("Unknown type: {:?}", token),
        });
        self.advance()?;
        output
    }

    /// ID (COMMA ID)* COLON type_spec
    fn variable_declaration(&mut self) -> anyhow::Result<Vec<Ast>> {
        let mut variable_names = vec![self.variable()?];
        while let Token::Comma = &self.current_token {
            self.advance()?;
            variable_names.push(self.variable()?);
        }
        eat!(self, Token::Colon);
        let type_spec = self.type_spec()?;
        let mut output = vec![];
        for var in variable_names {
            output.push(Ast::VariableDeclaration {
                variable: Box::from(var),
                type_spec: Box::from(Ast::Type(type_spec.clone())),
            })
        }
        Ok(output)
    }

    /// declarations : VAR (variable_declaration SEMI)+
    ///                | (PROCEDURE ID SEMI block SEMI)*
    //                 | empty
    fn declarations(&mut self) -> anyhow::Result<Vec<Ast>> {
        let mut declarations = vec![];
        while let Token::Keyword(Keyword::Var) = &self.current_token {
            self.advance()?;
            while let Token::Identifier(_) = &self.current_token {
                declarations.extend(self.variable_declaration()?);
                eat!(self, Token::Semi);
            }
        }
        while let Token::Keyword(Keyword::Procedure) = &self.current_token {
            self.advance()?;

            let procedure_name = self.variable()?;
            eat!(self, Token::Semi);

            let block_node = self.block()?;
            declarations.push(Ast::ProcedureDeclaration {
                name: procedure_name.variable()?.name.clone(),
                block: Box::from(block_node),
            });
            eat!(self, Token::Semi);
        }

        Ok(declarations)
    }

    /// block : declarations compound_statement
    fn block(&mut self) -> anyhow::Result<Ast> {
        Ok(Block {
            declarations: self.declarations()?,
            compound_statements: Box::from(self.compound_statement()?),
        })
    }

    /// program : PROGRAM variable SEMI block DOT
    fn program(&mut self) -> anyhow::Result<Ast> {
        eat!(self, Token::Keyword(Keyword::Program));
        let found_program_name = self.variable()?;
        let program_name = if let Ast::Variable(Variable { name }) = found_program_name {
            name
        } else {
            bail!("Expected a program name, but got {:?}", found_program_name)
        };

        eat!(self, Token::Semi);
        let block = self.block()?;
        eat!(self, Token::Dot);

        Ok(Program {
            name: program_name,
            block: Box::from(block),
        })
    }

    pub fn parse_expression(&mut self) -> anyhow::Result<Ast> {
        self.advance()?;
        self.expr()
    }

    pub fn parse(&mut self) -> anyhow::Result<Ast> {
        self.advance()?;
        let output = self.program()?;
        eat!(self, Token::Eof);

        Ok(output)
    }
}
