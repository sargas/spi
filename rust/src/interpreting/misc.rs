use crate::parsing::ast::Ast;

pub fn rpn(node: &Ast) -> String {
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
        Ast::ProcedureDeclaration { .. } => todo!(""),
        Ast::Parameter { .. } => todo!(""),
    }
}

pub fn lisp_notation(node: &Ast) -> String {
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
        Ast::ProcedureDeclaration { .. } => todo!(""),
        Ast::Parameter { .. } => todo!(""),
    }
}
