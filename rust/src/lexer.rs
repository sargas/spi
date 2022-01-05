use crate::Numeric;
use anyhow::{bail, Context, Ok, Result};
use std::iter;

#[derive(Debug)]
pub(crate) enum Token {
    Integer(Numeric),
    Plus,
    Minus,
    Multiply,
    Divide,
    ParenthesisStart,
    ParenthesisEnd,
    Eof,
}

pub(crate) struct Lexer {
    text: Vec<char>,
    pos: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub(crate) fn new(text: String) -> Lexer {
        Lexer {
            text: text.chars().collect(),
            pos: 0,
            current_char: text.chars().next(),
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
        if self.pos > self.text.len() - 1 {
            self.current_char = None;
        } else {
            self.current_char = Some(*self.text.get(self.pos).unwrap());
        }
    }

    fn integer(&mut self) -> Numeric {
        let mut num = String::from(self.current_char.unwrap());
        self.advance();
        while let Some(i) = self.current_char {
            if !i.is_numeric() {
                break;
            }
            num.push(i);
            self.advance();
        }
        num.parse::<Numeric>().unwrap()
    }

    fn get_next_token(&mut self) -> Result<Token> {
        if self.current_char.is_none() {
            return Ok(Token::Eof);
        }
        loop {
            let current_char = self
                .current_char
                .with_context(|| "Expecting another character")?;

            match current_char {
                ch if ch.is_whitespace() => {
                    self.advance();
                }
                ch if ch.is_numeric() => {
                    return Ok(Token::Integer(self.integer()));
                }
                '+' => {
                    self.advance();
                    return Ok(Token::Plus);
                }
                '-' => {
                    self.advance();
                    return Ok(Token::Minus);
                }
                '*' => {
                    self.advance();
                    return Ok(Token::Multiply);
                }
                '/' => {
                    self.advance();
                    return Ok(Token::Divide);
                }
                '(' => {
                    self.advance();
                    return Ok(Token::ParenthesisStart);
                }
                ')' => {
                    self.advance();
                    return Ok(Token::ParenthesisEnd);
                }
                ch => bail!("Unable to parse {:?}", ch),
            }
        }
    }

    pub(crate) fn parse(&mut self) -> Vec<Token> {
        let mut output = iter::from_fn(|| match self.get_next_token().ok() {
            Some(Token::Eof) => None,
            token => token,
        })
        .collect::<Vec<Token>>();
        output.extend([Token::Eof]); // re-add EOF
        output
    }
}
