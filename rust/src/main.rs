use crate::interpreter::{lisp_notation, rpn, Interpreter};
use crate::lexer::{Lexer, Token};
use crate::parser::Parser;
use anyhow::{Ok, Result};
use colored::*;
use std::io;
use std::io::{BufRead, Write};

mod interpreter;
mod lexer;
mod parser;

type Numeric = f64;

fn main() -> Result<()> {
    loop {
        print!("calc > ");
        io::stdout().flush()?;

        let stdin = io::stdin();
        let line = stdin.lock().lines().next().expect("could not read line")?;

        match line_to_result(line) {
            Result::Ok((result, ast_debug, rpn_output, lisp_output)) => {
                println!("{}: {}", "Result".green().bold(), result.to_string().bold());
                println!("AST: {}", ast_debug);
                println!("RPN: {}", rpn_output);
                println!("Lisp: {}", lisp_output);
                println!();
            }
            Err(err) => eprintln!("{}: {:?}", "Error: ".red(), err),
        }
    }
}

fn line_to_result(line: String) -> Result<(Numeric, String, String, String)> {
    let tokens = Lexer::new(line).parse();
    let ast = Parser::new(tokens).parse()?;

    Ok((
        Interpreter::new().visit(&ast),
        format!("{:?}", ast),
        rpn(&ast),
        lisp_notation(&ast),
    ))
}

// based on https://stackoverflow.com/a/34666891
macro_rules! interpreter_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() -> Result<()>{
            let (input, expected) = $value;

            let actual = line_to_result(input.to_owned())?.0;
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
    test_unary_operations: ("5 - - - + - (3 + 4) - +2", 10.0),
}
