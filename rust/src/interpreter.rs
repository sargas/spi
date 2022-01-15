use crate::parser::Ast;
use crate::Numeric;
use anyhow::{anyhow, bail, Result};
use case_insensitive_hashmap::CaseInsensitiveHashMap;

pub(crate) struct Interpreter {
    pub global_scope: CaseInsensitiveHashMap<Numeric>,
}

impl Interpreter {
    pub(crate) fn new() -> Interpreter {
        Interpreter {
            global_scope: CaseInsensitiveHashMap::new(),
        }
    }

    pub(crate) fn interpret(&mut self, node: &Ast) -> Result<()> {
        self.visit(node)
    }

    pub(crate) fn visit_expression(&self, node: &Ast) -> Result<Numeric> {
        Ok(match node {
            Ast::Add(l, r) => self.visit_expression(l)? + self.visit_expression(r)?,
            Ast::Subtract(l, r) => self.visit_expression(l)? - self.visit_expression(r)?,
            Ast::Multiply(l, r) => self.visit_expression(l)? * self.visit_expression(r)?,
            Ast::Divide(l, r) => self.visit_expression(l)? / self.visit_expression(r)?,
            Ast::Number(i) => *i,
            Ast::PositiveUnary(nested) => self.visit_expression(nested)?,
            Ast::NegativeUnary(nested) => -self.visit_expression(nested)?,
            Ast::Variable(var) => {
                *(self
                    .global_scope
                    .get(var.name.clone())
                    .ok_or_else(|| anyhow!("{:} not defined", var.name))?)
            }
            n => bail!("Invalid node in expression: {:?}", n),
        })
    }

    fn visit(&mut self, node: &Ast) -> Result<()> {
        match node {
            Ast::Compound { statements } => {
                for statement in statements {
                    self.visit(statement)?;
                }
            }
            Ast::Assign(var, expr) => {
                self.global_scope
                    .insert(var.name.clone(), self.visit_expression(expr)?);
            }
            Ast::NoOp => {}
            n => bail!("Invalid node in program: {:?}", n),
        }
        Ok(())
    }
}

pub(crate) fn rpn(node: &Ast) -> String {
    match node {
        Ast::Add(l, r) => format!("{} {} +", rpn(l), rpn(r)),
        Ast::Subtract(l, r) => format!("{} {} -", rpn(l), rpn(r)),
        Ast::Multiply(l, r) => format!("{} {} *", rpn(l), rpn(r)),
        Ast::Divide(l, r) => format!("{} {} /", rpn(l), rpn(r)),
        Ast::Number(i) => i.to_string(),
        Ast::PositiveUnary(nested) => rpn(nested),
        Ast::NegativeUnary(nested) => format!("0 {} -", rpn(nested)),
        Ast::Compound { .. } => todo!(""),
        Ast::Variable(_) => todo!(""),
        Ast::Assign(_, _) => todo!(""),
        Ast::NoOp => todo!(""),
    }
}

pub(crate) fn lisp_notation(node: &Ast) -> String {
    match node {
        Ast::Add(l, r) => format!("(+ {} {})", lisp_notation(l), lisp_notation(r)),
        Ast::Subtract(l, r) => format!("(- {} {})", lisp_notation(l), lisp_notation(r)),
        Ast::Multiply(l, r) => format!("(* {} {})", lisp_notation(l), lisp_notation(r)),
        Ast::Divide(l, r) => format!("(/ {} {})", lisp_notation(l), lisp_notation(r)),
        Ast::Number(i) => i.to_string(),
        Ast::PositiveUnary(nested) => lisp_notation(nested),
        Ast::NegativeUnary(nested) => format!("(- {})", lisp_notation(nested)),
        Ast::Compound { .. } => todo!(""),
        Ast::Variable(_) => todo!(""),
        Ast::Assign(_, _) => todo!(""),
        Ast::NoOp => todo!(""),
    }
}
