use std::io;
use std::io::{BufRead, Write};
use anyhow::Result;

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


enum Token {
    Initial,
    Integer(u32),
    Plus,
    Minus,
    EOF,
}

struct Interpreter {
    text: Vec<char>,
    pos: usize,
    current_token: Token,
}

impl Interpreter {
    fn new(text : String) -> Interpreter {
        Interpreter {
            text: text.chars().filter(|ch| !ch.is_whitespace()).collect(),
            pos: 0,
            current_token: Token::Initial,
        }
    }

    fn get_next_token(&self, pos: usize) -> Token {
        if pos > self.text.len() - 1 {
            return Token::EOF;
        }
        let current_char = self.text.get(pos);
        match current_char {
            Some(i) if i.is_numeric() => Token::Integer(i.to_digit(10).unwrap()),
            Some('+') => Token::Plus,
            Some('-') => Token::Minus,
            None => Token::EOF,
            _ => panic!("Error parsing")
        }
    }

    fn expr(&mut self) -> Result<String> {
        let left = self.get_next_token(0);
        let op = self.get_next_token(1);
        let right = self.get_next_token(2);

        match (left, op, right) {
            (Token::Integer(l), Token::Plus, Token::Integer(r)) => Ok((l + r).to_string()),
            (Token::Integer(l), Token::Minus, Token::Integer(r)) => Ok((l - r).to_string()),
            _ => panic!("eck"),
        }
    }
}
