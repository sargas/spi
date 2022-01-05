use crate::parser::Ast;
use crate::Numeric;

pub(crate) struct Interpreter {}

impl Interpreter {
    pub(crate) fn new() -> Interpreter {
        Interpreter {}
    }

    pub(crate) fn visit(&self, node: &Ast) -> Numeric {
        match node {
            Ast::Add(l, r) => self.visit(l) + self.visit(r),
            Ast::Subtract(l, r) => self.visit(l) - self.visit(r),
            Ast::Multiply(l, r) => self.visit(l) * self.visit(r),
            Ast::Divide(l, r) => self.visit(l) / self.visit(r),
            Ast::Number(i) => *i,
        }
    }
}
