use crate::{IntegerMachineType, RealMachineType};

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

#[derive(PartialEq, Debug, Clone)]
pub enum TypeSpec {
    Integer,
    Real,
}

#[derive(PartialEq, Debug)]
pub struct Variable {
    pub name: String,
}
