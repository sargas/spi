use crate::{IntegerMachineType, RealMachineType};
use anyhow::{bail, Result};

#[derive(PartialEq, Debug)]
pub enum Ast {
    Add(Box<Ast>, Box<Ast>),
    Subtract(Box<Ast>, Box<Ast>),
    Multiply(Box<Ast>, Box<Ast>),
    IntegerDivide(Box<Ast>, Box<Ast>),
    RealDivide(Box<Ast>, Box<Ast>),

    IntegerConstant(IntegerMachineType),
    RealConstant(RealMachineType),

    PositiveUnary(Box<Ast>),
    NegativeUnary(Box<Ast>),

    Program {
        name: String,
        block: Box<Ast>,
    },
    Block {
        declarations: Vec<Ast>,
        compound_statements: Box<Ast>,
    },
    ProcedureDeclaration {
        name: String,
        parameters: Vec<Ast>,
        block: Box<Ast>,
    },
    Parameter {
        variable: Box<Ast>,
        type_spec: Box<Ast>,
    },
    VariableDeclaration {
        variable: Box<Ast>,
        type_spec: Box<Ast>,
    },
    Type(TypeSpec),

    Compound {
        statements: Vec<Ast>,
    },
    Variable(Variable),
    Assign(Variable, Box<Ast>),
    NoOp,
}

impl Ast {
    pub fn variable(&self) -> Result<&Variable> {
        if let Ast::Variable(variable) = self {
            Ok(variable)
        } else {
            bail!("Expected a variable, was {:?}", self)
        }
    }
    pub fn type_spec(&self) -> Result<&TypeSpec> {
        if let Ast::Type(type_spec) = self {
            Ok(type_spec)
        } else {
            bail!("Expected a type spec, was {:?}", self)
        }
    }
}

#[derive(strum_macros::Display, PartialEq, Debug, Clone)]
pub enum TypeSpec {
    Integer,
    Real,
}

impl TypeSpec {
    pub(crate) fn to_ast_clone(&self) -> Ast {
        Ast::Type(self.clone())
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Variable {
    pub name: String,
}
