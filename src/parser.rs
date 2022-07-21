use std::fmt::Display;

use crate::{
    keywords::Keyword,
    lexer::{Lexem, Lexer, Token},
    literal::Literal,
};

#[derive(Debug, Clone)]
pub enum Expr {
    Expr {
        operator: Box<Expr>,
        operands: Vec<Expr>,
    },
    Var {
        name: String,
    },
    Literal(Literal),
    Keyword(Keyword),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Expr { operator, operands } => {
                let fmt_operands = operands
                    .iter()
                    .fold("".to_string(), |u, a| format!("{} {}", u, a));

                f.write_fmt(format_args!("({}{})", operator, fmt_operands))
            }
            Expr::Var { name } => f.write_fmt(format_args!("{}", name)),
            Expr::Literal(lit) => f.write_fmt(format_args!("{}", lit)),
            Expr::Keyword(keyword) => f.write_fmt(format_args!("{}", keyword)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseResult<T: Clone> {
    expr: T,
    next_position: usize,
}

impl<T: Clone> ParseResult<T> {
    pub fn expr(&self) -> &T {
        &self.expr
    }
}

macro_rules! token {
    ($tokens:expr, $position:expr) => {
        match $tokens.get($position) {
            Some(&(t, _)) => Ok(t),
            None => Err(format!(
                "Cannot parse an empty string [{}/{}]",
                $position,
                $tokens.len()
            )),
        }
    };
}

fn parse_keyword(tokens: &Vec<&Token>, position: usize) -> Result<ParseResult<Expr>, String> {
    match token!(tokens, position)? {
        Lexem::Keyword(keyword) => Ok(ParseResult {
            expr: Expr::Keyword(*keyword),
            next_position: position + 1,
        }),
        token => Err(format!("parse_keyword cannot parse {:?}", token)),
    }
}

fn parse_literal(tokens: &Vec<&Token>, position: usize) -> Result<ParseResult<Expr>, String> {
    match token!(tokens, position)? {
        Lexem::Literal(lit) => Ok(ParseResult {
            expr: Expr::Literal(lit.clone()),
            next_position: position + 1,
        }),
        token => Err(format!("parse_literal cannot parse {:?}", token)),
    }
}

fn parse_parenthesis_open(
    tokens: &Vec<&Token>,
    position: usize,
) -> Result<ParseResult<Lexem>, String> {
    match token!(tokens, position)? {
        Lexem::ParenthesisOpen => Ok(ParseResult {
            expr: Lexem::ParenthesisOpen,
            next_position: position + 1,
        }),
        token => Err(format!("parse_parenthesis_open cannot parse {:?}", token)),
    }
}

fn parse_parenthesis_close(
    tokens: &Vec<&Token>,
    position: usize,
) -> Result<ParseResult<Lexem>, String> {
    match token!(tokens, position)? {
        Lexem::ParenthesisClose => Ok(ParseResult {
            expr: Lexem::ParenthesisClose,
            next_position: position + 1,
        }),
        token => Err(format!("parse_parenthesis_close cannot parse {:?}", token)),
    }
}

fn parse_var(tokens: &Vec<&Token>, position: usize) -> Result<ParseResult<Expr>, String> {
    match token!(tokens, position)? {
        Lexem::Identifier(name) => Ok(ParseResult {
            expr: Expr::Var {
                name: name.to_owned(),
            },
            next_position: position + 1,
        }),
        token => Err(format!("parse_var cannot parse {:?}", token)),
    }
}

fn parse_operator(tokens: &Vec<&Token>, position: usize) -> Result<ParseResult<Expr>, String> {
    parse_var(tokens, position)
        .or_else(|_| parse_keyword(tokens, position))
        .or_else(|_| parse_expression(tokens, position))
        .or_else(|_| {
            Err(format!(
                "parse_operator cannot parse {:?}",
                tokens.get(position).unwrap()
            ))
        })
}

fn parse_list(tokens: &Vec<&Token>, position: usize) -> Result<ParseResult<Vec<Expr>>, String> {
    let mut expressions = vec![];
    let mut position = position;

    while let Ok(token) = token!(tokens, position) {
        match token {
            Lexem::ParenthesisClose => break,
            _ => {
                let ParseResult {
                    expr,
                    next_position,
                } = parse(tokens, position)?;

                position = next_position;
                expressions.push(expr);
            }
        }
    }

    Ok(ParseResult {
        expr: expressions,
        next_position: position + 1,
    })
}

fn parse_expression(tokens: &Vec<&Token>, position: usize) -> Result<ParseResult<Expr>, String> {
    parse_parenthesis_open(tokens, position).and_then(|r| {
        match parse_operator(tokens, r.next_position) {
            Ok(ParseResult {
                expr: operator,
                next_position,
            }) => {
                let ParseResult {
                    expr: operands,
                    next_position,
                } = parse_list(tokens, next_position)?;

                match &operator {
                    Expr::Expr {
                        operator: op,
                        operands: op_operands,
                    } => {
                        let mut new_operands = vec![];
                        new_operands.extend(op_operands.into_iter().map(|x| x.clone()));
                        new_operands.extend(operands.into_iter());

                        Ok(ParseResult {
                            expr: Expr::Expr {
                                operator: op.clone(),
                                operands: new_operands,
                            },
                            next_position: next_position,
                        })
                    }
                    _ => Ok(ParseResult {
                        expr: Expr::Expr {
                            operator: Box::new(operator),
                            operands,
                        },
                        next_position,
                    }),
                }
            }
            Err(err) => {
                let ParseResult {
                    expr: operands,
                    next_position,
                } = parse_list(tokens, r.next_position)?;

                if operands.is_empty() {
                    Err(err)
                } else {
                    Ok(ParseResult {
                        expr: Expr::Expr {
                            operator: Box::new(Expr::Keyword(Keyword::Id)),
                            operands,
                        },
                        next_position,
                    })
                }
            }
        }
    })
}

fn parse(tokens: &Vec<&Token>, position: usize) -> Result<ParseResult<Expr>, String> {
    parse_keyword(tokens, position)
        .or_else(|_| parse_literal(tokens, position))
        .or_else(|_| parse_var(tokens, position))
        .or_else(|_| parse_expression(tokens, position))
}

pub fn run_parser(tokens: &Vec<&Token>) -> Result<ParseResult<Expr>, String> {
    if tokens.is_empty() {
        Ok(ParseResult {
            expr: Expr::Literal(Literal::Nil),
            next_position: 1,
        })
    } else {
        parse(tokens, 0)
    }
}

#[macro_export]
macro_rules! parse {
    ($src:expr) => {{
        let mut lexer = crate::lexer::Lexer::new();

        lexer.lex(&$src.to_string()).unwrap();

        let r = crate::parser::run_parser(&lexer.lexems()).unwrap();

        r.expr().clone()
    }};
}
