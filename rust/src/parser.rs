use crate::lexer::Keyword;
use crate::{Numeric, Token};
use anyhow::{bail, Result};

#[cfg(test)]
use crate::Lexer;

#[derive(PartialEq, Debug)]
pub(crate) enum Ast {
    Add(Box<Ast>, Box<Ast>),
    Subtract(Box<Ast>, Box<Ast>),
    Multiply(Box<Ast>, Box<Ast>),
    Divide(Box<Ast>, Box<Ast>),

    Number(Numeric),

    PositiveUnary(Box<Ast>),
    NegativeUnary(Box<Ast>),

    Compound { statements: Vec<Ast> },
    Variable(Variable),
    Assign(Variable, Box<Ast>),
    NoOp,
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

    /// factor : (PLUS | MINUS) factor | INTEGER | LPAREN expr RPAREN | variable
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
            Token::Integer(i) => {
                self.advance()?;
                Ok(Ast::Number(i))
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

    /// term : factor ((MUL | DIV) factor)*
    fn term(&mut self) -> Result<Ast> {
        let mut result = self.factor()?;

        loop {
            match self.current_token {
                Token::Multiply => {
                    self.advance()?;
                    result = Ast::Multiply(Box::from(result), Box::from(self.factor()?));
                }
                Token::Keyword(Keyword::Div) => {
                    self.advance()?;
                    result = Ast::Divide(Box::from(result), Box::from(self.factor()?));
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

    /// program : compound_statement DOT
    fn program(&mut self) -> Result<Ast> {
        let output = self.compound_statement();
        match &self.current_token {
            Token::Dot => self.advance()?,
            t => bail!("Expected a dot, found {:?}", t),
        };
        output
    }

    pub(crate) fn parse_expression(&mut self) -> Result<Ast> {
        self.advance()?;
        self.expr()
    }

    pub(crate) fn parse(&mut self) -> Result<Ast> {
        self.advance()?;
        let output = self.program();
        match &self.current_token {
            Token::Eof => {}
            t => bail!("Expected the end of the file, found {:?}", t),
        };

        output
    }
}

#[test]
fn test_simple() -> Result<()> {
    assert_eq!(
        Parser::new(vec![Ok(Token::Integer(4)), Ok(Token::Eof)].into_iter()).parse_expression()?,
        Ast::Number(4),
    );
    Ok(())
}

#[test]
fn test_one_operation() -> Result<()> {
    assert_eq!(
        Parser::new(
            vec![
                Ok(Token::Integer(4)),
                Ok(Token::Plus),
                Ok(Token::Integer(6)),
                Ok(Token::Eof)
            ]
            .into_iter()
        )
        .parse_expression()?,
        Ast::Add(Box::from(Ast::Number(4)), Box::from(Ast::Number(6))),
    );
    Ok(())
}

#[test]
fn test_multiple_operations() -> Result<()> {
    assert_eq!(
        Parser::new(
            vec![
                Ok(Token::Integer(1)),
                Ok(Token::Plus),
                Ok(Token::Integer(2)),
                Ok(Token::Plus),
                Ok(Token::Integer(3)),
                Ok(Token::Plus),
                Ok(Token::Integer(4)),
                Ok(Token::Eof)
            ]
            .into_iter()
        )
        .parse_expression()?,
        Ast::Add(
            Box::from(Ast::Add(
                Box::from(Ast::Add(
                    Box::from(Ast::Number(1)),
                    Box::from(Ast::Number(2))
                )),
                Box::from(Ast::Number(3))
            )),
            Box::from(Ast::Number(4))
        ),
    );
    Ok(())
}

#[test]
fn test_overriding_precedence() -> Result<()> {
    assert_eq!(
        Parser::new(
            vec![
                Ok(Token::Integer(1)),
                Ok(Token::Multiply),
                Ok(Token::ParenthesisStart),
                Ok(Token::Integer(2)),
                Ok(Token::Plus),
                Ok(Token::Integer(3)),
                Ok(Token::Multiply),
                Ok(Token::Integer(4)),
                Ok(Token::ParenthesisEnd),
                Ok(Token::Eof)
            ]
            .into_iter()
        )
        .parse_expression()?,
        Ast::Multiply(
            Box::from(Ast::Number(1)),
            Box::from(Ast::Add(
                Box::from(Ast::Number(2)),
                Box::from(Ast::Multiply(
                    Box::from(Ast::Number(3)),
                    Box::from(Ast::Number(4))
                ))
            ))
        ),
    );
    Ok(())
}

#[test]
fn test_program() -> Result<()> {
    let code = r#"BEGIN
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
        Ast::Compound {
            statements: vec![
                Ast::Compound {
                    statements: vec![
                        Ast::Assign(
                            Variable {
                                name: "number".to_string()
                            },
                            Box::from(Ast::Number(2))
                        ),
                        Ast::Assign(
                            Variable {
                                name: "a".to_string()
                            },
                            Box::from(Ast::Variable(Variable {
                                name: "number".to_string()
                            }))
                        ),
                        //
                        //
                        //
                        Ast::Assign(
                            Variable {
                                name: "b".to_string()
                            },
                            Box::from(Ast::Add(
                                Box::from(Ast::Multiply(
                                    Box::from(Ast::Number(10)),
                                    Box::from(Ast::Variable(Variable {
                                        name: "a".to_string()
                                    }))
                                )),
                                Box::from(Ast::Divide(
                                    Box::from(Ast::Multiply(
                                        Box::from(Ast::Number(10)),
                                        Box::from(Ast::Variable(Variable {
                                            name: "number".to_string()
                                        }))
                                    )),
                                    Box::from(Ast::Number(4)),
                                ))
                            ))
                        ),
                        Ast::Assign(
                            Variable {
                                name: "c".to_string()
                            },
                            Box::from(Ast::Subtract(
                                Box::from(Ast::Variable(Variable {
                                    name: "a".to_string()
                                })),
                                Box::from(Ast::NegativeUnary(Box::from(Ast::Variable(Variable {
                                    name: "b".to_string()
                                }))))
                            ))
                        ),
                    ]
                },
                Ast::Assign(
                    Variable {
                        name: "x".to_string()
                    },
                    Box::from(Ast::Number(11))
                ),
                Ast::NoOp,
            ]
        }
    );

    Ok(())
}
