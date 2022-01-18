use crate::lexing::lexer::Lexer;
use crate::lexing::token::Token;
use crate::parsing::ast::{Ast, TypeSpec, Variable};
use crate::parsing::parser::Parser;

#[test]
fn test_simple() -> anyhow::Result<()> {
    assert_eq!(
        Parser::new(vec![Ok(Token::IntegerConstant(4)), Ok(Token::Eof)].into_iter())
            .parse_expression()?,
        Ast::IntegerConstant(4),
    );
    Ok(())
}

#[test]
fn test_one_operation() -> anyhow::Result<()> {
    assert_eq!(
        Parser::new(
            vec![
                Ok(Token::IntegerConstant(4)),
                Ok(Token::Plus),
                Ok(Token::IntegerConstant(6)),
                Ok(Token::Eof),
            ]
            .into_iter()
        )
        .parse_expression()?,
        Ast::Add(
            Box::from(Ast::IntegerConstant(4)),
            Box::from(Ast::IntegerConstant(6)),
        ),
    );
    Ok(())
}

#[test]
fn test_multiple_operations() -> anyhow::Result<()> {
    assert_eq!(
        Parser::new(
            vec![
                Ok(Token::IntegerConstant(1)),
                Ok(Token::Plus),
                Ok(Token::IntegerConstant(2)),
                Ok(Token::Plus),
                Ok(Token::IntegerConstant(3)),
                Ok(Token::Plus),
                Ok(Token::IntegerConstant(4)),
                Ok(Token::Eof),
            ]
            .into_iter()
        )
        .parse_expression()?,
        Ast::Add(
            Box::from(Ast::Add(
                Box::from(Ast::Add(
                    Box::from(Ast::IntegerConstant(1)),
                    Box::from(Ast::IntegerConstant(2)),
                )),
                Box::from(Ast::IntegerConstant(3)),
            )),
            Box::from(Ast::IntegerConstant(4)),
        ),
    );
    Ok(())
}

#[test]
fn test_overriding_precedence() -> anyhow::Result<()> {
    assert_eq!(
        Parser::new(
            vec![
                Ok(Token::IntegerConstant(1)),
                Ok(Token::Multiply),
                Ok(Token::ParenthesisStart),
                Ok(Token::IntegerConstant(2)),
                Ok(Token::Plus),
                Ok(Token::IntegerConstant(3)),
                Ok(Token::Multiply),
                Ok(Token::IntegerConstant(4)),
                Ok(Token::ParenthesisEnd),
                Ok(Token::Eof),
            ]
            .into_iter()
        )
        .parse_expression()?,
        Ast::Multiply(
            Box::from(Ast::IntegerConstant(1)),
            Box::from(Ast::Add(
                Box::from(Ast::IntegerConstant(2)),
                Box::from(Ast::Multiply(
                    Box::from(Ast::IntegerConstant(3)),
                    Box::from(Ast::IntegerConstant(4)),
                )),
            )),
        ),
    );
    Ok(())
}

#[test]
fn test_program() -> anyhow::Result<()> {
    let code = r#"PROGRAM test; BEGIN
            BEGIN
                number := 2;
                a := number;
                b := 10 * a + 10 * number div 4;
                c := a - - b
            END;
            x := 11;
        END."#;
    let result = Parser::new(Lexer::new(code)).parse()?;

    assert_eq!(
        result,
        Ast::Program {
            name: "test".to_string(),
            block: Box::from(Ast::Block {
                declarations: vec![],
                compound_statements: Box::from(Ast::Compound {
                    statements: vec![
                        Ast::Compound {
                            statements: vec![
                                Ast::Assign(
                                    Variable {
                                        name: "number".to_string()
                                    },
                                    Box::from(Ast::IntegerConstant(2)),
                                ),
                                Ast::Assign(
                                    Variable {
                                        name: "a".to_string()
                                    },
                                    Box::from(Ast::Variable(Variable {
                                        name: "number".to_string()
                                    })),
                                ),
                                Ast::Assign(
                                    Variable {
                                        name: "b".to_string()
                                    },
                                    Box::from(Ast::Add(
                                        Box::from(Ast::Multiply(
                                            Box::from(Ast::IntegerConstant(10)),
                                            Box::from(Ast::Variable(Variable {
                                                name: "a".to_string()
                                            })),
                                        )),
                                        Box::from(Ast::IntegerDivide(
                                            Box::from(Ast::Multiply(
                                                Box::from(Ast::IntegerConstant(10)),
                                                Box::from(Ast::Variable(Variable {
                                                    name: "number".to_string()
                                                })),
                                            )),
                                            Box::from(Ast::IntegerConstant(4)),
                                        )),
                                    )),
                                ),
                                Ast::Assign(
                                    Variable {
                                        name: "c".to_string()
                                    },
                                    Box::from(Ast::Subtract(
                                        Box::from(Ast::Variable(Variable {
                                            name: "a".to_string()
                                        })),
                                        Box::from(Ast::NegativeUnary(Box::from(Ast::Variable(
                                            Variable {
                                                name: "b".to_string()
                                            }
                                        )))),
                                    )),
                                ),
                            ]
                        },
                        Ast::Assign(
                            Variable {
                                name: "x".to_string()
                            },
                            Box::from(Ast::IntegerConstant(11)),
                        ),
                        Ast::NoOp,
                    ]
                }),
            }),
        },
    );

    Ok(())
}

#[test]
fn test_program2() {
    let code = r#"
            PROGRAM Part10AST;
        VAR
           a, b : INTEGER;
           y    : REAL;

        BEGIN {Part10AST}
           a := 2;
           b := 10 * a + 10 * a DIV 4;
           y := 20 / 7 + 3.14;
        END.  {Part10AST}
    "#;
    let result = Parser::new(Lexer::new(code)).parse().unwrap();
    assert_eq!(
        Ast::Program {
            name: "Part10AST".to_string(),
            block: Box::from(Ast::Block {
                declarations: vec![
                    Ast::VariableDeclaration {
                        variable: Box::from(Ast::Variable(Variable {
                            name: "a".to_string()
                        })),
                        type_spec: Box::from(Ast::Type(TypeSpec::Integer))
                    },
                    Ast::VariableDeclaration {
                        variable: Box::from(Ast::Variable(Variable {
                            name: "b".to_string()
                        })),
                        type_spec: Box::from(Ast::Type(TypeSpec::Integer))
                    },
                    Ast::VariableDeclaration {
                        variable: Box::from(Ast::Variable(Variable {
                            name: "y".to_string()
                        })),
                        type_spec: Box::from(Ast::Type(TypeSpec::Real))
                    },
                ],
                compound_statements: Box::from(Ast::Compound {
                    statements: vec![
                        Ast::Assign(
                            Variable {
                                name: "a".to_string()
                            },
                            Box::from(Ast::IntegerConstant(2))
                        ),
                        Ast::Assign(
                            Variable {
                                name: "b".to_string()
                            },
                            Box::from(Ast::Add(
                                Box::from(Ast::Multiply(
                                    Box::from(Ast::IntegerConstant(10)),
                                    Box::from(Ast::Variable(Variable {
                                        name: "a".to_string()
                                    }))
                                )),
                                Box::from(Ast::IntegerDivide(
                                    Box::from(Ast::Multiply(
                                        Box::from(Ast::IntegerConstant(10)),
                                        Box::from(Ast::Variable(Variable {
                                            name: "a".to_string()
                                        }))
                                    )),
                                    Box::from(Ast::IntegerConstant(4))
                                ))
                            ))
                        ),
                        Ast::Assign(
                            Variable {
                                name: "y".to_string()
                            },
                            Box::from(Ast::Add(
                                Box::from(Ast::RealDivide(
                                    Box::from(Ast::IntegerConstant(20)),
                                    Box::from(Ast::IntegerConstant(7))
                                )),
                                Box::from(Ast::RealConstant(3.14))
                            ))
                        ),
                        Ast::NoOp,
                    ]
                })
            })
        },
        result
    );
}
