use anyhow::{Context, Ok, Result};
use clap::Parser as ClapParser;
use cli_table::format::Justify;
use cli_table::{print_stdout, Cell, Style, Table};
use colored::*;
use spi::interpreting::interpreter::Interpreter;
use spi::interpreting::misc::{lisp_notation, rpn};
use spi::interpreting::symbol_table::SymbolTable;
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
    #[clap(short('t'), long)]
    show_tree: bool,

    /// Show Symbol Table Debug Info
    #[clap(short('s'), long)]
    show_symbols: bool,

    /// Show everything
    #[clap(short('a'), long)]
    show_all: bool,
}

fn main() -> Result<()> {
    let args: CliArgs = CliArgs::parse();

    if args.path.is_some() {
        let path = args.path.unwrap();
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("could not read file `{}`", &path.to_string_lossy()))?;

        let tokens = Lexer::new(&content);
        let ast = Parser::new(tokens).parse()?;
        let mut interpreter = Interpreter::new(args.show_symbols || args.show_all);
        let output = interpreter.interpret(&ast);

        if args.show_tree || args.show_all {
            println!("Tree:\n{:#?}", ast);
            println!("\n");
        }
        if args.show_symbols || args.show_all {
            display_symbol_table(&interpreter.symbol_table.unwrap())?;
        }
        println!("\nVariables:");
        print_stdout(
            interpreter
                .global_scope
                .iter()
                .map(|(key, value)| {
                    vec![
                        key.to_string().cell().bold(true),
                        value.to_string().cell().justify(Justify::Right),
                    ]
                })
                .table()
                .title(vec![
                    "Variables".cell().bold(true),
                    "Value".cell().bold(true),
                ]),
        )?;
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
        Interpreter::new(false).interpret_expression(&ast)?,
        format!("{:?}", ast),
        rpn(&ast),
        lisp_notation(&ast),
    ))
}

fn display_symbol_table(symbol_table: &SymbolTable) -> std::io::Result<()> {
    println!("\nSymbol Table:\n");
    println!("Scope Name: {}", symbol_table.scope_name);
    println!("Scope Level: {}", symbol_table.scope_level);

    print_stdout(
        symbol_table
            .symbols
            .iter()
            .map(|(key, symbol)| {
                vec![
                    key.to_string().cell().bold(true),
                    symbol.to_string().cell().justify(Justify::Right),
                ]
            })
            .table()
            .title(vec!["Name".cell().bold(true), "Symbol".cell().bold(true)]),
    )
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
