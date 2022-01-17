use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Mul, Neg, Sub};
use crate::parser::Ast;
use crate::{IntegerMachineType, RealMachineType};
use anyhow::{anyhow, bail, Result};
use case_insensitive_hashmap::CaseInsensitiveHashMap;

pub(crate) struct Interpreter {
    pub global_scope: CaseInsensitiveHashMap<NumericType>,
}

impl Interpreter {
    pub(crate) fn new() -> Interpreter {
        Interpreter {
            global_scope: CaseInsensitiveHashMap::new(),
        }
    }

    pub(crate) fn interpret_expression(&self, node: &Ast) -> Result<NumericType> {
        Ok(match node {
            Ast::Add(l, r) => self.interpret_expression(l)? + self.interpret_expression(r)?,
            Ast::Subtract(l, r) => self.interpret_expression(l)? - self.interpret_expression(r)?,
            Ast::Multiply(l, r) => self.interpret_expression(l)? * self.interpret_expression(r)?,
            Ast::IntegerDivide(l, r) => {
                NumericType::Integer(self.interpret_expression(l)?.as_int() / self.interpret_expression(r)?.as_int())
            }
            Ast::IntegerConstant(i) => NumericType::Integer(*i),
            Ast::RealDivide(l, r) => {
                NumericType::Real(self.interpret_expression(l)?.as_real() / self.interpret_expression(r)?.as_real())
            }
            Ast::RealConstant(r) => NumericType::Real(*r),
            Ast::PositiveUnary(nested) => self.interpret_expression(nested)?,
            Ast::NegativeUnary(nested) => -self.interpret_expression(nested)?,
            Ast::Variable(var) => {
                *(self
                    .global_scope
                    .get(var.name.clone())
                    .ok_or_else(|| anyhow!("{:} not defined", var.name))?)
            }
            Ast::Compound { .. }
            | Ast::Assign(_, _)
            | Ast::Program { .. }
            | Ast::Block { .. }
            | Ast::VariableDeclaration { .. }
            | Ast::Type(_)
            | Ast::NoOp => {
                bail!("Invalid node in expression: {:?}", node)
            }
        })
    }

    pub(crate) fn interpret(&mut self, node: &Ast) -> Result<()> {
        match node {
            Ast::Compound { statements } => {
                for statement in statements {
                    self.interpret(statement)?;
                }
            }
            Ast::Assign(var, expr) => {
                self.global_scope
                    .insert(var.name.clone(), self.interpret_expression(expr)?);
            }
            Ast::NoOp => {}
            Ast::Program { block, .. } => self.interpret(block)?,
            Ast::Block {
                declarations,
                compound_statements,
            } => {
                for variable_declaration in declarations {
                    self.interpret(variable_declaration)?;
                }
                self.interpret(compound_statements)?;
            }
            // TODO for type safety
            Ast::VariableDeclaration { .. } => {}
            Ast::Type(_) => {}

            Ast::Add(_, _)
            | Ast::Subtract(_, _)
            | Ast::Multiply(_, _)
            | Ast::IntegerDivide(_, _)
            | Ast::IntegerConstant(_)
            | Ast::RealDivide(_, _)
            | Ast::RealConstant(_)
            | Ast::PositiveUnary(_)
            | Ast::NegativeUnary(_)
            | Ast::Variable(_) => bail!("Invalid node in program: {:?}", node),
        }
        Ok(())
    }
}

#[derive(Clone,Copy,Debug,PartialEq)]
pub(crate) enum NumericType {
    Integer(IntegerMachineType),
    Real(RealMachineType),
}

impl NumericType {
    fn as_real(&self) -> RealMachineType {
        match self {
            NumericType::Integer(i) => *i as RealMachineType,
            NumericType::Real(r) => *r,
        }
    }
    fn as_int(&self) -> IntegerMachineType {
        match self {
            NumericType::Integer(i) => *i,
            NumericType::Real(r) => *r as IntegerMachineType,
        }
    }
}

impl Display for NumericType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NumericType::Integer(i) => Display::fmt(&i, f),
            NumericType::Real(r) => Display::fmt(&r, f),
        }
    }
}

impl Add for NumericType {
    type Output = NumericType;

    fn add(self, rhs: Self) -> Self::Output {
        if let (NumericType::Integer(i1), NumericType::Integer(i2)) = (self, rhs) {
            NumericType::Integer(i1 + i2)
        } else {
            NumericType::Real(self.as_real() + rhs.as_real())
        }
    }
}

impl Sub for NumericType {
    type Output = NumericType;

    fn sub(self, rhs: Self) -> Self::Output {
        if let (NumericType::Integer(i1), NumericType::Integer(i2)) = (self, rhs) {
            NumericType::Integer(i1 - i2)
        } else {
            NumericType::Real(self.as_real() - rhs.as_real())
        }
    }
}

impl Mul for NumericType {
    type Output = NumericType;

    fn mul(self, rhs: Self) -> Self::Output {
        if let (NumericType::Integer(i1), NumericType::Integer(i2)) = (self, rhs) {
            NumericType::Integer(i1 * i2)
        } else {
            NumericType::Real(self.as_real() * rhs.as_real())
        }
    }
}

impl Neg for NumericType {
    type Output = NumericType;

    fn neg(self) -> Self::Output {
        match self {
            NumericType::Integer(i) => NumericType::Integer(-i),
            NumericType::Real(r) => NumericType::Real(-r),
        }
    }
}

pub(crate) fn rpn(node: &Ast) -> String {
    match node {
        Ast::Add(l, r) => format!("{} {} +", rpn(l), rpn(r)),
        Ast::Subtract(l, r) => format!("{} {} -", rpn(l), rpn(r)),
        Ast::Multiply(l, r) => format!("{} {} *", rpn(l), rpn(r)),
        Ast::IntegerDivide(l, r) => format!("{} {} /", rpn(l), rpn(r)),
        Ast::IntegerConstant(i) => i.to_string(),
        Ast::PositiveUnary(nested) => rpn(nested),
        Ast::NegativeUnary(nested) => format!("0 {} -", rpn(nested)),
        Ast::Compound { .. } => todo!(""),
        Ast::Variable(_) => todo!(""),
        Ast::Assign(_, _) => todo!(""),
        Ast::NoOp => todo!(""),
        Ast::RealDivide(_, _) => todo!(""),
        Ast::RealConstant(_) => todo!(""),
        Ast::Program { .. } => todo!(""),
        Ast::Block { .. } => todo!(""),
        Ast::VariableDeclaration { .. } => todo!(""),
        Ast::Type(_) => todo!(""),
    }
}

pub(crate) fn lisp_notation(node: &Ast) -> String {
    match node {
        Ast::Add(l, r) => format!("(+ {} {})", lisp_notation(l), lisp_notation(r)),
        Ast::Subtract(l, r) => format!("(- {} {})", lisp_notation(l), lisp_notation(r)),
        Ast::Multiply(l, r) => format!("(* {} {})", lisp_notation(l), lisp_notation(r)),
        Ast::IntegerDivide(l, r) => format!("(/ {} {})", lisp_notation(l), lisp_notation(r)),
        Ast::IntegerConstant(i) => i.to_string(),
        Ast::PositiveUnary(nested) => lisp_notation(nested),
        Ast::NegativeUnary(nested) => format!("(- {})", lisp_notation(nested)),
        Ast::Compound { .. } => todo!(""),
        Ast::Variable(_) => todo!(""),
        Ast::Assign(_, _) => todo!(""),
        Ast::NoOp => todo!(""),
        Ast::RealDivide(_, _) => todo!(""),
        Ast::RealConstant(_) => todo!(""),
        Ast::Program { .. } => todo!(""),
        Ast::Block { .. } => todo!(""),
        Ast::VariableDeclaration { .. } => todo!(""),
        Ast::Type(_) => todo!(""),
    }
}
