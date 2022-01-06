use crate::{Numeric, Token};
use anyhow::{anyhow, bail, Result};

#[derive(PartialEq, Debug)]
pub(crate) enum Ast {
    Add(Box<Ast>, Box<Ast>),
    Subtract(Box<Ast>, Box<Ast>),
    Multiply(Box<Ast>, Box<Ast>),
    Divide(Box<Ast>, Box<Ast>),

    Number(Numeric),

    PositiveUnary(Box<Ast>),
    NegativeUnary(Box<Ast>),
}

pub(crate) struct Parser<I: Iterator<Item=Result<Token>>> {
    current_token: Token,
    tokens: I,
}

impl<I: Iterator<Item=Result<Token>>> Parser<I> {
    pub(crate) fn new(tokens: I) -> Parser<I> {
        Parser {
            current_token: Token::Eof,
            tokens,
        }
    }


    fn advance(&mut self) -> Result<()> {
        self.current_token = self.tokens.next().ok_or(anyhow!("no tokens left"))??;
        Ok(())
    }

    /// factor : (PLUS | MINUS) factor | INTEGER | LPAREN expr RPAREN
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
            _ => bail!(
                "Expected integer or parenthesis instead of {:?}",
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
                Token::Divide => {
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

    pub(crate) fn parse(&mut self) -> Result<Ast> {
        self.advance()?;
        self.expr()
    }
}

#[test]
fn test_simple() -> Result<()> {
    assert_eq!(
        Parser::new(vec![Ok(Token::Integer(4.0)), Ok(Token::Eof)].into_iter()).parse()?,
        Ast::Number(4.0),
    );
    Ok(())
}

#[test]
fn test_one_operation() -> Result<()> {
    assert_eq!(
        Parser::new(vec![
            Ok(Token::Integer(4.0)),
               Ok(Token::Plus),
                  Ok(Token::Integer(6.0)),
                     Ok(Token::Eof)
        ].into_iter())
        .parse()?,
        Ast::Add(Box::from(Ast::Number(4.0)), Box::from(Ast::Number(6.0))),
    );
    Ok(())
}

#[test]
fn test_multiple_operations() -> Result<()> {
    assert_eq!(
        Parser::new(vec![
            Ok(Token::Integer(1.0)),
            Ok(Token::Plus),
            Ok(Token::Integer(2.0)),
            Ok(Token::Plus),
            Ok(Token::Integer(3.0)),
            Ok(Token::Plus),
            Ok(Token::Integer(4.0)),
            Ok(Token::Eof)
        ].into_iter())
        .parse()?,
        Ast::Add(
            Box::from(Ast::Add(
                Box::from(Ast::Add(
                    Box::from(Ast::Number(1.0)),
                    Box::from(Ast::Number(2.0))
                )),
                Box::from(Ast::Number(3.0))
            )),
            Box::from(Ast::Number(4.0))
        ),
    );
    Ok(())
}

#[test]
fn test_overriding_precedence() -> Result<()> {
    assert_eq!(
        Parser::new(vec![
            Ok(Token::Integer(1.0)),
            Ok(Token::Multiply),
            Ok(Token::ParenthesisStart),
            Ok(Token::Integer(2.0)),
            Ok(Token::Plus),
            Ok(Token::Integer(3.0)),
            Ok(Token::Multiply),
            Ok(Token::Integer(4.0)),
            Ok(Token::ParenthesisEnd),
            Ok(Token::Eof)
        ].into_iter())
        .parse()?,
        Ast::Multiply(
            Box::from(Ast::Number(1.0)),
            Box::from(Ast::Add(
                Box::from(Ast::Number(2.0)),
                Box::from(Ast::Multiply(
                    Box::from(Ast::Number(3.0)),
                    Box::from(Ast::Number(4.0))
                ))
            ))
        ),
    );
    Ok(())
}
