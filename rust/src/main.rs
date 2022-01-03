use std::{io, iter};
use std::io::{BufRead, Write};
use anyhow::{anyhow, bail, Result, Ok};

fn main() -> Result<()> {
    loop {
        print!("calc > ");
        io::stdout().flush()?;

        let stdin = io::stdin();
        let line = stdin.lock().lines().next().expect("could not read line")?;
        let mut interpreter = Interpreter::new(line);
        println!("{}", interpreter.expr()?);
    }
}

#[derive(Debug)]
enum Token {
    Integer(u32),
    Plus,
    Minus,
    Multiply,
    Divide,
    EOF,
}

struct Interpreter {
    current_token: Token,
    tokens: std::vec::IntoIter<Token>,
}

impl Interpreter {
    fn new(text : String) -> Interpreter {
        Interpreter {
            current_token: Token::EOF,
            tokens: Lexer::new(text).parse().into_iter(),
        }
    }

    fn expr(&mut self) -> Result<String> {
        self.advance()?;
        let mut result = self.term()?;

        loop {
            match self.current_token {
                Token::Plus => {
                    self.advance()?;
                    result += self.term()?;
                }
                Token::Minus => {
                    self.advance()?;
                    result -= self.term()?;
                }
                Token::Multiply => {
                    self.advance()?;
                    result *= self.term()?;
                }
                Token::Divide => {
                    self.advance()?;
                    result /= self.term()?;
                }
                Token::EOF => {
                    break;
                }
                _ => {
                    return Err(anyhow!("invalid token: {:?}", self.current_token));
                }
            }
        }

        Ok(result.to_string())
    }

    fn advance(&mut self) -> Result<()>{
        self.current_token = self.tokens.next().ok_or(anyhow!("no tokens left"))?;
        Ok(())
    }

    fn term(&mut self) -> Result<u32> {
        if let Token::Integer(i) = self.current_token {
            self.advance()?;
            Ok(i)
        } else {
            bail!("unknown term {:?}", self.current_token);
        }
    }
}

struct Lexer {
    text: Vec<char>,
    pos: usize,
    current_char: Option<char>,
}

impl Lexer {
    fn new(text: String) -> Lexer {
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

    fn integer(&mut self) -> u32 {
        let mut num = String::from(self.current_char.unwrap());
        self.advance();
        while let Some(i) = self.current_char {
            if !i.is_numeric() { break; }
            num.push(i);
            self.advance();
        }
        num.parse::<u32>().unwrap()
    }

    fn get_next_token(&mut self) -> Result<Token> {
        if let None = self.current_char {
            return Ok(Token::EOF);
        }
        loop {
            let current_char = self.current_char.unwrap();
            match current_char {
                ch if ch.is_whitespace() => {
                    self.advance();
                },
                ch if ch.is_numeric() => {
                    return Ok(Token::Integer(self.integer()));
                },
                '+' => {
                    self.advance();
                    return Ok(Token::Plus)
                },
                '-' => {
                    self.advance();
                    return Ok(Token::Minus)
                },
                '*' => {
                    self.advance();
                    return Ok(Token::Multiply)
                },
                '/' => {
                    self.advance();
                    return Ok(Token::Divide)
                },
                ch => return Err(anyhow!("Unable to parse {:?}", ch)),
            }
        }
    }

    fn parse(&mut self) -> Vec<Token> {
        let mut output = iter::from_fn(|| {
            match self.get_next_token().ok() {
                Some(Token::EOF) => None,
                token => token,
            }})
            .collect::<Vec<Token>>();
        output.extend([Token::EOF]); // re-add EOF
        output
    }
}
