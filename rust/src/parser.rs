use crate::lexer::Keyword;
use crate::{IntegerMachineType, RealMachineType, Token};
use anyhow::{bail, Result};

use crate::parser::Ast::{Block, Program};
#[cfg(test)]
use crate::Lexer;

#[derive(PartialEq, Debug)]
pub(crate) enum Ast {
    Add(Box<Ast>, Box<Ast>),
    Subtract(Box<Ast>, Box<Ast>),
    Multiply(Box<Ast>, Box<Ast>),
    IntegerDivide(Box<Ast>, Box<Ast>),
    RealDivide(Box<Ast>, Box<Ast>),

    IntegerConstant(IntegerMachineType),
    RealConstant(RealMachineType),

    PositiveUnary(Box<Ast>),
    NegativeUnary(Box<Ast>),

    Program {
        name: String,
        block: Box<Ast>,
    },
    Block {
        declarations: Vec<Ast>,
        compound_statements: Box<Ast>,
    },
    VariableDeclaration {
        variable: Box<Ast>,
        type_spec: Box<Ast>,
    },
    Type(TypeSpec),

    Compound {
        statements: Vec<Ast>,
    },
    Variable(Variable),
    Assign(Variable, Box<Ast>),
    NoOp,
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum TypeSpec {
    Integer,
    Real,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Variable {
    pub name: String,
}

pub(crate) struct Parser<I: Iterator<Item = Result<Token>>> {
    current_token: Token,
    tokens: I,
}

impl<I: Iterator<Item = Result<Token>>> Parser<I> {
    pub(crate) fn new(tokens: I) -> Parser<I> {
        Parser {
            current_token: Token::Eof,
            tokens,
        }
    }

    fn advance(&mut self) -> Result<()> {
        self.current_token = self
            .tokens
            .next()
            .unwrap_or(Ok(Token::Eof))
            .unwrap_or(Token::Eof);
        Ok(())
    }

    /// factor : (PLUS | MINUS) factor | INTEGER_CONST | REAL_CONST | LPAREN expr RPAREN | variable
    fn factor(&mut self) -> Result<Ast> {
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
                if let Token::ParenthesisEnd = self.current_token {
                    self.advance()?;
                    nested_result
                } else {
                    bail!("Expected ')' instead of {:?}", self.current_token)
                }
            }
            Token::Identifier(_) => self.variable(),
            _ => bail!(
                "Expected integer, parenthesis, or variable instead of {:?}",
                self.current_token
            ),
        }
    }

    /// term : factor ((MUL | INTEGER_DIV | REAL_DIV) factor)*
    fn term(&mut self) -> Result<Ast> {
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

    fn expr(&mut self) -> Result<Ast> {
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
    fn empty(&mut self) -> Result<Ast> {
        Ok(Ast::NoOp)
    }

    /// variable : ID
    fn variable(&mut self) -> Result<Ast> {
        if let Token::Identifier(variable_name) = &self.current_token {
            let name = variable_name.clone();
            self.advance()?;
            Ok(Ast::Variable(Variable { name }))
        } else {
            bail!("Expected a variable, found {:?}", self.current_token)
        }
    }

    /// assignment_statement : variable ASSIGN expr
    fn assignment_statement(&mut self) -> Result<Ast> {
        let var_node = self.variable()?;

        match &self.current_token {
            Token::Assign => self.advance()?,
            t => bail!("Expected assignment operator, found {:?}", t),
        };
        let variable = match var_node {
            Ast::Variable(variable) => variable,
            _ => panic!("Parser.variable() returned something that isn't a variable!"),
        };
        Ok(Ast::Assign(variable, Box::from(self.expr()?)))
    }

    /// statement : compound_statement
    ///               | assignment_statement
    ///               | empty
    fn statement(&mut self) -> Result<Ast> {
        match &self.current_token {
            Token::Keyword(Keyword::Begin) => self.compound_statement(),
            Token::Identifier(_) => self.assignment_statement(),
            _ => self.empty(),
        }
    }

    /// statement_list : statement
    ///                    | statement SEMI statement_list
    fn statement_list(&mut self) -> Result<Vec<Ast>> {
        let mut statements = vec![self.statement()?];
        while let &Token::Semi = &self.current_token {
            self.advance()?;
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    /// compound_statement: BEGIN statement_list END
    fn compound_statement(&mut self) -> Result<Ast> {
        match &self.current_token {
            Token::Keyword(Keyword::Begin) => self.advance()?,
            t => bail!("Expected BEGIN, found {:?}", t),
        };
        let statements = self.statement_list()?;
        match &self.current_token {
            Token::Keyword(Keyword::End) => self.advance()?,
            t => bail!("Expected END, found {:?}", t),
        };

        Ok(Ast::Compound { statements })
    }

    /// type_spec : INTEGER | REAL
    fn type_spec(&mut self) -> Result<TypeSpec> {
        let output = Ok(match &self.current_token {
            Token::Keyword(Keyword::Integer) => TypeSpec::Integer,
            Token::Keyword(Keyword::Real) => TypeSpec::Real,
            token => bail!("Unknown type: {:?}", token),
        });
        self.advance()?;
        output
    }

    /// ID (COMMA ID)* COLON type_spec
    fn variable_declaration(&mut self) -> Result<Vec<Ast>> {
        let mut variable_names = vec![self.variable()?];
        while let Token::Comma = &self.current_token {
            self.advance()?;
            variable_names.push(self.variable()?);
        }
        match &self.current_token {
            Token::Colon => self.advance()?,
            t => bail!("Expected a colon, found {:?}", t),
        }
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
    //                     | empty
    fn declarations(&mut self) -> Result<Vec<Ast>> {
        let mut declarations = vec![];
        if let Token::Keyword(Keyword::Var) = &self.current_token {
            self.advance()?;
            while let Token::Identifier(_) = &self.current_token {
                declarations.extend(self.variable_declaration()?);
                match &self.current_token {
                    Token::Semi => self.advance()?,
                    t => bail!("Expected a Semicolon, found {:?}", t),
                };
            }
        }

        Ok(declarations)
    }

    /// block : declarations compound_statement
    fn block(&mut self) -> Result<Ast> {
        Ok(Block {
            declarations: self.declarations()?,
            compound_statements: Box::from(self.compound_statement()?),
        })
    }

    /// program : PROGRAM variable SEMI block DOT
    fn program(&mut self) -> Result<Ast> {
        match &self.current_token {
            Token::Keyword(Keyword::Program) => self.advance()?,
            t => bail!("Expected 'PROGRAM', found {:?}", t),
        };
        let found_program_name = self.variable()?;
        let program_name = if let Ast::Variable(Variable { name }) = found_program_name {
            name
        } else {
            bail!("Expected a program name, but got {:?}", found_program_name)
        };

        match &self.current_token {
            Token::Semi => self.advance()?,
            t => bail!("Expected ';', found {:?}", t),
        };
        let block = self.block()?;

        match &self.current_token {
            Token::Dot => self.advance()?,
            t => bail!("Expected a dot, found {:?}", t),
        };
        Ok(Program {
            name: program_name,
            block: Box::from(block),
        })
    }

    pub(crate) fn parse_expression(&mut self) -> Result<Ast> {
        self.advance()?;
        self.expr()
    }

    pub(crate) fn parse(&mut self) -> Result<Ast> {
        self.advance()?;
        let output = self.program()?;
        match &self.current_token {
            Token::Eof => {}
            t => bail!("Expected the end of the file, found {:?}", t),
        };

        Ok(output)
    }
}

#[test]
fn test_simple() -> Result<()> {
    assert_eq!(
        Parser::new(vec![Ok(Token::IntegerConstant(4)), Ok(Token::Eof)].into_iter())
            .parse_expression()?,
        Ast::IntegerConstant(4),
    );
    Ok(())
}

#[test]
fn test_one_operation() -> Result<()> {
    assert_eq!(
        Parser::new(
            vec![
                Ok(Token::IntegerConstant(4)),
                Ok(Token::Plus),
                Ok(Token::IntegerConstant(6)),
                Ok(Token::Eof),
            ]
            .into_iter()
        )
        .parse_expression()?,
        Ast::Add(
            Box::from(Ast::IntegerConstant(4)),
            Box::from(Ast::IntegerConstant(6)),
        ),
    );
    Ok(())
}

#[test]
fn test_multiple_operations() -> Result<()> {
    assert_eq!(
        Parser::new(
            vec![
                Ok(Token::IntegerConstant(1)),
                Ok(Token::Plus),
                Ok(Token::IntegerConstant(2)),
                Ok(Token::Plus),
                Ok(Token::IntegerConstant(3)),
                Ok(Token::Plus),
                Ok(Token::IntegerConstant(4)),
                Ok(Token::Eof),
            ]
            .into_iter()
        )
        .parse_expression()?,
        Ast::Add(
            Box::from(Ast::Add(
                Box::from(Ast::Add(
                    Box::from(Ast::IntegerConstant(1)),
                    Box::from(Ast::IntegerConstant(2)),
                )),
                Box::from(Ast::IntegerConstant(3)),
            )),
            Box::from(Ast::IntegerConstant(4)),
        ),
    );
    Ok(())
}

#[test]
fn test_overriding_precedence() -> Result<()> {
    assert_eq!(
        Parser::new(
            vec![
                Ok(Token::IntegerConstant(1)),
                Ok(Token::Multiply),
                Ok(Token::ParenthesisStart),
                Ok(Token::IntegerConstant(2)),
                Ok(Token::Plus),
                Ok(Token::IntegerConstant(3)),
                Ok(Token::Multiply),
                Ok(Token::IntegerConstant(4)),
                Ok(Token::ParenthesisEnd),
                Ok(Token::Eof),
            ]
            .into_iter()
        )
        .parse_expression()?,
        Ast::Multiply(
            Box::from(Ast::IntegerConstant(1)),
            Box::from(Ast::Add(
                Box::from(Ast::IntegerConstant(2)),
                Box::from(Ast::Multiply(
                    Box::from(Ast::IntegerConstant(3)),
                    Box::from(Ast::IntegerConstant(4)),
                )),
            )),
        ),
    );
    Ok(())
}

#[test]
fn test_program() -> Result<()> {
    let code = r#"PROGRAM test; BEGIN
            BEGIN
                number := 2;
                a := number;
                b := 10 * a + 10 * number div 4;
                c := a - - b
            END;
            x := 11;
        END."#;
    let result = Parser::new(Lexer::new(code)).parse()?;

    assert_eq!(
        result,
        Ast::Program {
            name: "test".to_string(),
            block: Box::from(Ast::Block {
                declarations: vec![],
                compound_statements: Box::from(Ast::Compound {
                    statements: vec![
                        Ast::Compound {
                            statements: vec![
                                Ast::Assign(
                                    Variable {
                                        name: "number".to_string()
                                    },
                                    Box::from(Ast::IntegerConstant(2)),
                                ),
                                Ast::Assign(
                                    Variable {
                                        name: "a".to_string()
                                    },
                                    Box::from(Ast::Variable(Variable {
                                        name: "number".to_string()
                                    })),
                                ),
                                Ast::Assign(
                                    Variable {
                                        name: "b".to_string()
                                    },
                                    Box::from(Ast::Add(
                                        Box::from(Ast::Multiply(
                                            Box::from(Ast::IntegerConstant(10)),
                                            Box::from(Ast::Variable(Variable {
                                                name: "a".to_string()
                                            })),
                                        )),
                                        Box::from(Ast::IntegerDivide(
                                            Box::from(Ast::Multiply(
                                                Box::from(Ast::IntegerConstant(10)),
                                                Box::from(Ast::Variable(Variable {
                                                    name: "number".to_string()
                                                })),
                                            )),
                                            Box::from(Ast::IntegerConstant(4)),
                                        )),
                                    )),
                                ),
                                Ast::Assign(
                                    Variable {
                                        name: "c".to_string()
                                    },
                                    Box::from(Ast::Subtract(
                                        Box::from(Ast::Variable(Variable {
                                            name: "a".to_string()
                                        })),
                                        Box::from(Ast::NegativeUnary(Box::from(Ast::Variable(
                                            Variable {
                                                name: "b".to_string()
                                            }
                                        )))),
                                    )),
                                ),
                            ]
                        },
                        Ast::Assign(
                            Variable {
                                name: "x".to_string()
                            },
                            Box::from(Ast::IntegerConstant(11)),
                        ),
                        Ast::NoOp,
                    ]
                }),
            }),
        },
    );

    Ok(())
}

#[test]
fn test_program2() {
    let code = r#"
            PROGRAM Part10AST;
        VAR
           a, b : INTEGER;
           y    : REAL;

        BEGIN {Part10AST}
           a := 2;
           b := 10 * a + 10 * a DIV 4;
           y := 20 / 7 + 3.14;
        END.  {Part10AST}
    "#;
    let result = Parser::new(Lexer::new(code)).parse().unwrap();
    assert_eq!(
        Ast::Program {
            name: "Part10AST".to_string(),
            block: Box::from(Ast::Block {
                declarations: vec![
                    Ast::VariableDeclaration {
                        variable: Box::from(Ast::Variable(Variable {
                            name: "a".to_string()
                        })),
                        type_spec: Box::from(Ast::Type(TypeSpec::Integer))
                    },
                    Ast::VariableDeclaration {
                        variable: Box::from(Ast::Variable(Variable {
                            name: "b".to_string()
                        })),
                        type_spec: Box::from(Ast::Type(TypeSpec::Integer))
                    },
                    Ast::VariableDeclaration {
                        variable: Box::from(Ast::Variable(Variable {
                            name: "y".to_string()
                        })),
                        type_spec: Box::from(Ast::Type(TypeSpec::Real))
                    },
                ],
                compound_statements: Box::from(Ast::Compound {
                    statements: vec![
                        Ast::Assign(
                            Variable {
                                name: "a".to_string()
                            },
                            Box::from(Ast::IntegerConstant(2))
                        ),
                        Ast::Assign(
                            Variable {
                                name: "b".to_string()
                            },
                            Box::from(Ast::Add(
                                Box::from(Ast::Multiply(
                                    Box::from(Ast::IntegerConstant(10)),
                                    Box::from(Ast::Variable(Variable {
                                        name: "a".to_string()
                                    }))
                                )),
                                Box::from(Ast::IntegerDivide(
                                    Box::from(Ast::Multiply(
                                        Box::from(Ast::IntegerConstant(10)),
                                        Box::from(Ast::Variable(Variable {
                                            name: "a".to_string()
                                        }))
                                    )),
                                    Box::from(Ast::IntegerConstant(4))
                                ))
                            ))
                        ),
                        Ast::Assign(
                            Variable {
                                name: "y".to_string()
                            },
                            Box::from(Ast::Add(
                                Box::from(Ast::RealDivide(
                                    Box::from(Ast::IntegerConstant(20)),
                                    Box::from(Ast::IntegerConstant(7))
                                )),
                                Box::from(Ast::RealConstant(3.14))
                            ))
                        ),
                        Ast::NoOp,
                    ]
                })
            })
        },
        result
    );
}
