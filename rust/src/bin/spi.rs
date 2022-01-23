use anyhow::{Context, Ok, Result};
use clap::Parser as ClapParser;
use colored::*;
use spi::interpreting::interpreter::Interpreter;
use spi::interpreting::misc::{lisp_notation, rpn};
use spi::interpreting::types::NumericType;
use spi::lexing::lexer::Lexer;
use spi::parsing::parser::Parser;
use std::io;
use std::io::{BufRead, Write};

#[derive(ClapParser)]
#[clap(author, version, about)]
struct CliArgs {
    /// Pascal file to interpret
    #[clap(parse(from_os_str))]
    path: Option<std::path::PathBuf>,

    /// Show the AST
    #[clap(short)]
    show_tree: bool,
}

fn main() -> Result<()> {
    let args: CliArgs = CliArgs::parse();

    if args.path.is_some() {
        let path = args.path.unwrap();
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("could not read file `{}`", &path.to_string_lossy()))?;

        let tokens = Lexer::new(&content);
        let ast = Parser::new(tokens).parse()?;
        let mut interpreter = Interpreter::new();
        let output = interpreter.interpret(&ast);

        if args.show_tree {
            println!("Tree:\n{:#?}", ast);
            println!("\n");
        }
        println!("Symbol Table:\n{:#?}", interpreter.symbol_table);
        println!("Variables:\n{:#?}", interpreter.global_scope);
        return output;
    }

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

fn line_to_result(line: String) -> Result<(NumericType, String, String, String)> {
    let tokens = Lexer::new(&line);
    let ast = Parser::new(tokens).parse_expression()?;

    Ok((
        Interpreter::new().interpret_expression(&ast)?,
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
    test_simple_int: ("4", NumericType::Integer(4)),
    test_addition: ("1 + 4", NumericType::Integer(5)),
    test_multiple_operators: ("1 + 3 * 5", NumericType::Integer(16)),
    test_parenthesis: ("(1 + 3) * 5", NumericType::Integer(20)),
    test_nested_parenthesis: ("7 + 3 * (10 div (12 Div (3 + 1) - 1)) dIV (2 + 3) - 5 - 3 + (8)", NumericType::Integer(10)),
    test_unary_operations: ("5 - - - + - (3 + 4) - +2", NumericType::Integer(10)),
}
