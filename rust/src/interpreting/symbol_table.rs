use crate::parsing::ast::Ast;
use anyhow::{bail, Result};
use case_insensitive_hashmap::CaseInsensitiveHashMap;
use std::fmt::{Display, Formatter};
use std::string::ToString;
use strum_macros::Display;

#[derive(Debug)]
pub enum Symbol {
    BuiltIn(BuiltInTypes),
    Variable { name: String, var_type: String },
}

#[derive(Display, Debug)]
pub enum BuiltInTypes {
    Integer,
    Real,
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::BuiltIn(x) => x.fmt(f),
            Symbol::Variable { name, var_type } => format!("<{}:{}>", name, var_type).fmt(f),
        }
    }
}

impl Symbol {
    fn symbol_table_key(&self) -> String {
        match self {
            Symbol::BuiltIn(x) => x.to_string(),
            Symbol::Variable { name, .. } => name.clone(),
        }
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    pub symbols: CaseInsensitiveHashMap<Symbol>,
    verbose: bool,
}

impl SymbolTable {
    fn define(&mut self, symbol: Symbol) -> Result<()> {
        if self.verbose {
            println!("Define: {}", symbol);
        }
        let key = symbol.symbol_table_key();
        if self.symbols.contains_key(key) {
            bail!("Duplicate identifier: {:?}", symbol)
        } else {
            self.symbols.insert(symbol.symbol_table_key(), symbol);
            Ok(())
        }
    }

    fn lookup(&self, name: &str) -> Option<&Symbol> {
        if self.verbose {
            println!("Lookup: {}", name);
        }
        self.symbols.get(name)
    }
}

impl SymbolTable {
    pub(crate) fn build_for(program: &Ast, verbose: bool) -> Result<SymbolTable> {
        let mut symbol_table = SymbolTable {
            symbols: CaseInsensitiveHashMap::new(),
            verbose,
        };

        symbol_table.define(Symbol::BuiltIn(BuiltInTypes::Integer))?;
        symbol_table.define(Symbol::BuiltIn(BuiltInTypes::Real))?;

        let result = build_symbol_table(&mut symbol_table, program);

        result.and(Ok(symbol_table))
    }
}

fn build_symbol_table(symbols: &mut SymbolTable, node: &Ast) -> Result<()> {
    match node {
        Ast::Add(l, r)
        | Ast::Subtract(l, r)
        | Ast::Multiply(l, r)
        | Ast::IntegerDivide(l, r)
        | Ast::RealDivide(l, r) => {
            build_symbol_table(symbols, l).and_then(|_| build_symbol_table(symbols, r))
        }
        Ast::IntegerConstant(_) | Ast::RealConstant(_) => Ok(()),
        Ast::PositiveUnary(node) => build_symbol_table(symbols, node),
        Ast::NegativeUnary(node) => build_symbol_table(symbols, node),
        Ast::Program { block, .. } => build_symbol_table(symbols, block),
        Ast::ProcedureDeclaration { .. } => Ok(()), // TODO after part 12
        Ast::Block {
            declarations,
            compound_statements,
        } => {
            let declaration_results: Result<()> = declarations
                .iter()
                .try_for_each(|declaration| build_symbol_table(symbols, declaration));

            declaration_results.and_then(|_| build_symbol_table(symbols, compound_statements))
        }
        Ast::VariableDeclaration {
            variable,
            type_spec: type_spec_node,
        } => {
            let variable_type = type_spec_node.type_spec()?.to_string();
            if symbols.lookup(&variable_type).is_none() {
                bail!("Unknown type: {:?}", variable);
            }
            let name = variable.variable()?.name.clone();
            if symbols.lookup(&name).is_some() {
                bail!("Duplicate Identifier: {:?}", variable);
            }
            symbols.define(Symbol::Variable {
                name,
                var_type: variable_type,
            })?;
            Ok(())
        }
        Ast::Compound { statements } => statements
            .iter()
            .try_for_each(|statement| build_symbol_table(symbols, statement)),
        Ast::Assign(variable, expr) => {
            build_symbol_table(symbols, expr)?;
            if symbols.lookup(&variable.name).is_none() {
                bail!("Unknown variable to assign to: {:?}", variable);
            }
            Ok(())
        }
        Ast::Variable(variable) => {
            if symbols.lookup(&variable.name).is_none() {
                bail!("Unknown variable: {:?}", variable);
            }
            Ok(())
        }
        Ast::Type(_) | Ast::NoOp => Ok(()),
    }
}

#[test]
fn test_part11() {
    let code = r#"
        PROGRAM Part11;
        VAR
           number : INTEGER;
           a, b   : INTEGER;
           y      : REAL;

        BEGIN {Part11}
           number := 2;
           a := number ;
           b := 10 * a + 10 * number DIV 4;
           y := 20 / 7 + 3.14
        END.  {Part11}
    "#;

    use crate::lexing::lexer::Lexer;
    use crate::parsing::parser::Parser;
    let ast = Parser::new(Lexer::new(code)).parse().unwrap();
    assert!(SymbolTable::build_for(&ast, true).is_ok());
}

#[test]
fn test_part13_happy_path() {
    let code = r#"
        program SymTab4;
        var x, y : integer;

        begin
            x := x + y;
        end.
    "#;

    use crate::lexing::lexer::Lexer;
    use crate::parsing::parser::Parser;
    let ast = Parser::new(Lexer::new(code)).parse().unwrap();
    assert!(SymbolTable::build_for(&ast, true).is_ok());
}

#[test]
fn test_part13_sym_tab_5() {
    let code = r#"
        program SymTab5;
        var x : integer;

        begin
            x := y;
        end.
    "#;

    use crate::lexing::lexer::Lexer;
    use crate::parsing::parser::Parser;
    let ast = Parser::new(Lexer::new(code)).parse().unwrap();
    assert!(SymbolTable::build_for(&ast, true)
        .expect_err("Expected not to find y")
        .to_string()
        .contains("Unknown variable"));
}

#[test]
fn test_duplicate_identifiers() {
    let code = r#"
        program SymTab6;
        var x, y : integer;
        var y : real;
        begin
            x := x + y;
        end.
    "#;

    use crate::lexing::lexer::Lexer;
    use crate::parsing::parser::Parser;
    let ast = Parser::new(Lexer::new(code)).parse().unwrap();
    assert!(SymbolTable::build_for(&ast, true)
        .expect_err("Expected y to be defined twice")
        .to_string()
        .contains("Duplicate Identifier"));
}
