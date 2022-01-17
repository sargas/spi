use crate::{IntegerMachineType, RealMachineType};
use anyhow::{bail, Context, Ok, Result};
use std::str::FromStr;
use strum_macros::EnumString;

#[derive(Debug, PartialEq)]
pub(crate) enum Token {
    IntegerConstant(IntegerMachineType),
    RealConstant(RealMachineType),
    Plus,
    Minus,
    Multiply,
    RealDivision,
    ParenthesisStart,
    ParenthesisEnd,
    Eof,
    Keyword(Keyword),
    Identifier(String),
    Semi,
    Assign,
    Dot,
    Colon,
    Comma,
}

#[derive(Debug, EnumString, PartialEq)]
#[strum(ascii_case_insensitive)]
pub(crate) enum Keyword {
    Begin,
    End,
    #[strum(serialize = "div")]
    IntegerDiv,
    Var,
    Integer,
    Real,
    Program,
}

pub(crate) struct Lexer {
    text: Vec<char>,
    pos: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub(crate) fn new(text: &str) -> Lexer {
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

    fn integer(&mut self) -> String {
        let mut num = String::from(self.current_char.unwrap());
        self.advance();
        while let Some(i) = self.current_char {
            if !i.is_numeric() {
                break;
            }
            num.push(i);
            self.advance();
        }
        num
    }

    fn constant_number(&mut self) -> Token {
        let mut num = self.integer();

        if let Some('.') = self.current_char {
            num.push_str(&self.integer());
            Token::RealConstant(num.parse::<RealMachineType>().unwrap())
        } else {
            Token::IntegerConstant(num.parse::<IntegerMachineType>().unwrap())
        }
    }

    fn id(&mut self) -> String {
        let mut name = String::new();

        // Allow for starting underscore
        if let Some('_') = self.current_char {
            name.push('_');
            self.advance();
        }

        while self.current_char.filter(|c| c.is_alphanumeric()).is_some() {
            name.push(self.current_char.unwrap());
            self.advance();
        }
        name
    }

    fn skip_until_comment_ends(&mut self) {
        let mut current_char = self.current_char;
        while current_char.unwrap() != '}' {
            self.advance();
            current_char = self.current_char;
        }
        self.advance(); // skip }
    }

    fn peek(&self) -> Option<&char> {
        self.text.get(self.pos + 1)
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
                '{' => {
                    self.advance();
                    self.skip_until_comment_ends();
                }
                ch if ch.is_numeric() => {
                    return Ok(self.constant_number());
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
                    return Ok(Token::RealDivision);
                }
                '(' => {
                    self.advance();
                    return Ok(Token::ParenthesisStart);
                }
                ')' => {
                    self.advance();
                    return Ok(Token::ParenthesisEnd);
                }
                ch if ch.is_alphabetic() || '_' == ch => {
                    let name = self.id();
                    return match Keyword::from_str(&name) {
                        std::result::Result::Ok(keyword) => Ok(Token::Keyword(keyword)),
                        _ => Ok(Token::Identifier(name)),
                    };
                }
                ':' if self.peek().filter(|ch| *ch == &'=').is_some() => {
                    self.advance();
                    self.advance();
                    return Ok(Token::Assign);
                }
                ':' => {
                    self.advance();
                    return Ok(Token::Colon);
                }
                ';' => {
                    self.advance();
                    return Ok(Token::Semi);
                }
                '.' => {
                    self.advance();
                    return Ok(Token::Dot);
                }
                ',' => {
                    self.advance();
                    return Ok(Token::Comma);
                }
                ch => bail!("Unable to parse {:?}", ch),
            }
        }
    }
}

impl Iterator for Lexer {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.get_next_token())
    }
}

#[test]
fn test_lexer() -> Result<()> {
    let expected_tokens = vec![
        Token::Keyword(Keyword::Begin),
        Token::Identifier("a".to_string()),
        Token::Assign,
        Token::IntegerConstant(2),
        Token::Semi,
        Token::Identifier("_num".to_string()),
        Token::Assign,
        Token::Identifier("a".to_string()),
        Token::Multiply,
        Token::RealConstant(5.0),
        Token::Semi,
        Token::Keyword(Keyword::End),
        Token::Dot,
    ];

    let lexer = Lexer::new("BEGIN a := 2; _num := a * 5.0; END.");
    for (actual, expected) in lexer.zip(expected_tokens) {
        assert_eq!(actual?, expected);
    }
    Ok(())
}
