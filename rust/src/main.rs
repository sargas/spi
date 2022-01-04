use std::{io, iter};
use std::io::{BufRead, Write};
use anyhow::{anyhow, bail, Result, Ok, Context};
use colored::*;

type Numeric = f64;

fn main() -> Result<()> {
    loop {
        print!("calc > ");
        io::stdout().flush()?;

        let stdin = io::stdin();
        let line = stdin.lock().lines().next().expect("could not read line")?;
        let mut interpreter = Interpreter::new(line);
        match interpreter.interpret() {
            Result::Ok(result) => println!("{}", result),
            Err(err) => eprintln!("{}: {:?}", "Error: ".red(), err),
        }
    }
}

#[derive(Debug)]
enum Token {
    Integer(Numeric),
    Plus,
    Minus,
    Multiply,
    Divide,
    ParenthesisStart,
    ParenthesisEnd,
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

    /// Arithmetic expression interpreter.
    ///
    ///         calc>  14 + 2 * 3 - 6 / 2
    ///         17
    ///
    ///         expr   : term ((PLUS | MINUS) term)*
    ///         term   : factor ((MUL | DIV) factor)*
    ///         factor : INTEGER | LPAREN expr RPAREN
    fn interpret(&mut self) -> Result<Numeric> {
        self.advance()?;
        self.expr()
    }

    fn expr(&mut self) -> Result<Numeric> {
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
                _ => {
                    break;
                }
            }
        }

        Ok(result)
    }

    fn advance(&mut self) -> Result<()>{
        self.current_token = self.tokens.next().ok_or(anyhow!("no tokens left"))?;
        Ok(())
    }


    fn factor(&mut self) -> Result<Numeric> {
        match self.current_token {
            Token::Integer(i) => {
                self.advance()?;
                Ok(i)
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
            _ => bail!("Expected integer or parenthesis instead of {:?}", self.current_token)
        }
    }

    fn term(&mut self) -> Result<Numeric> {
        let mut result = self.factor()?;

        loop {
            match self.current_token {
                Token::Multiply => {
                    self.advance()?;
                    result *= self.term()?;
                }
                Token::Divide => {
                    self.advance()?;
                    result /= self.term()?;
                }
                _ => {
                    break;
                }
            }
        }
        Ok(result)
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

    fn integer(&mut self) -> Numeric {
        let mut num = String::from(self.current_char.unwrap());
        self.advance();
        while let Some(i) = self.current_char {
            if !i.is_numeric() { break; }
            num.push(i);
            self.advance();
        }
        num.parse::<Numeric>().unwrap()
    }

    fn get_next_token(&mut self) -> Result<Token> {
        if let None = self.current_char {
            return Ok(Token::EOF);
        }
        loop {
            let current_char = self.current_char
                .with_context(|| "Expecting another character")?;

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
                '(' => {
                    self.advance();
                    return Ok(Token::ParenthesisStart)
                },
                ')' => {
                    self.advance();
                    return Ok(Token::ParenthesisEnd)
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

// based on https://stackoverflow.com/a/34666891
macro_rules! interpreter_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() -> Result<()>{
            let (input, expected) = $value;

            let mut interpreter = Interpreter::new(input.to_owned());
            let actual = interpreter.interpret()?;
            assert_eq!(actual, expected);
            Ok(())
        }
    )*
    }
}
interpreter_tests! {
    test_simple_int: ("4", 4.0),
    test_addition: ("1 + 4", 5.0),
    test_multiple_operators: ("1 + 3 * 5", 16.0),
    test_parenthesis: ("(1 + 3) * 5", 20.0),
    test_nested_parenthesis: ("7 + 3 * (10 / (12 / (3 + 1) - 1)) / (2 + 3) - 5 - 3 + (8)", 10.0),
}
