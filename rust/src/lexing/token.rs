use crate::{IntegerMachineType, RealMachineType};
use strum_macros::EnumString;

#[derive(Debug, PartialEq)]
pub enum Token {
    IntegerConstant(IntegerMachineType),
    RealConstant(RealMachineType),
    Plus,
    Minus,
    Multiply,
    RealDivision,
    ParenthesisStart,
    ParenthesisEnd,
    Eof,
    Keyword(Keyword),
    Identifier(String),
    Semi,
    Assign,
    Dot,
    Colon,
    Comma,
}

#[derive(Debug, EnumString, PartialEq)]
#[strum(ascii_case_insensitive)]
pub enum Keyword {
    Begin,
    End,
    #[strum(serialize = "div")]
    IntegerDiv,
    Var,
    Integer,
    Real,
    Program,
}
