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

pub(crate) fn rpn(node: &Ast) -> String {
    match node {
        Ast::Add(l, r) => format!("{} {} +", rpn(l), rpn(r)),
        Ast::Subtract(l, r) => format!("{} {} -", rpn(l), rpn(r)),
        Ast::Multiply(l, r) => format!("{} {} *", rpn(l), rpn(r)),
        Ast::Divide(l, r) => format!("{} {} /", rpn(l), rpn(r)),
        Ast::Number(i) => i.to_string(),
    }
}

pub(crate) fn lisp_notation(node: &Ast) -> String {
    match node {
        Ast::Add(l, r) => format!("(+ {} {})", lisp_notation(l), lisp_notation(r)),
        Ast::Subtract(l, r) => format!("(- {} {})", lisp_notation(l), lisp_notation(r)),
        Ast::Multiply(l, r) => format!("(* {} {})", lisp_notation(l), lisp_notation(r)),
        Ast::Divide(l, r) => format!("(/ {} {})", lisp_notation(l), lisp_notation(r)),
        Ast::Number(i) => i.to_string(),
    }
}
