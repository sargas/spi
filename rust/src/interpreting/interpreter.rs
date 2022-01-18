use crate::interpreting::types::NumericType;
use crate::parsing::ast::Ast;
use anyhow::{anyhow, bail};
use case_insensitive_hashmap::CaseInsensitiveHashMap;

pub struct Interpreter {
    pub global_scope: CaseInsensitiveHashMap<NumericType>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            global_scope: CaseInsensitiveHashMap::new(),
        }
    }

    pub fn interpret_expression(&self, node: &Ast) -> anyhow::Result<NumericType> {
        Ok(match node {
            Ast::Add(l, r) => self.interpret_expression(l)? + self.interpret_expression(r)?,
            Ast::Subtract(l, r) => self.interpret_expression(l)? - self.interpret_expression(r)?,
            Ast::Multiply(l, r) => self.interpret_expression(l)? * self.interpret_expression(r)?,
            Ast::IntegerDivide(l, r) => NumericType::Integer(
                self.interpret_expression(l)?.as_int() / self.interpret_expression(r)?.as_int(),
            ),
            Ast::IntegerConstant(i) => NumericType::Integer(*i),
            Ast::RealDivide(l, r) => NumericType::Real(
                self.interpret_expression(l)?.as_real() / self.interpret_expression(r)?.as_real(),
            ),
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

    pub fn interpret(&mut self, node: &Ast) -> anyhow::Result<()> {
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

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
