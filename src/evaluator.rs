use std::collections::{HashMap, VecDeque};

use crate::{
    frame::Frame,
    keywords::Keyword,
    parser::{Expr, ParseResult},
};

#[derive(Debug)]
pub struct Runtime {
    stack: VecDeque<Frame>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            stack: VecDeque::new(),
        }
    }

    fn lookup(&mut self, name: &String) -> Option<&Expr> {
        self.stack
            .iter()
            .fold(None, |u, frame| frame.lookup(name).or(u))
    }

    fn push_var(&mut self, name: String, value: Expr) {
        if let Some(frame) = self.stack.back_mut() {
            frame.push(name, value);
        } else {
            self.stack.push_back(Frame::new(name, value));
        }
    }

    pub fn replace_free(&self, name: &String, value: &Expr, ast: Expr) -> Expr {
        let new_expr = match &ast {
            Expr::Expr { operator, operands } => match *operator.clone() {
                Expr::Var { name: v_name } if v_name == *name => Expr::Expr {
                    operator: Box::new(value.clone()),
                    operands: operands
                        .iter()
                        .map(|operand| self.replace_free(name, value, operand.clone()))
                        .collect::<Vec<Expr>>(),
                },
                Expr::Keyword(kw) => match &kw {
                    Keyword::Def | Keyword::Lambda | Keyword::External => match operands.get(0) {
                        Some(Expr::Var { name: v_name }) if v_name == name => ast,
                        _ => Expr::Expr {
                            operator: Box::new(Expr::Keyword(kw)),
                            operands: operands
                                .iter()
                                .map(|operand| self.replace_free(name, value, operand.clone()))
                                .collect::<Vec<Expr>>(),
                        },
                    },
                    _ => ast,
                },
                op => Expr::Expr {
                    operator: Box::new(op),
                    operands: operands
                        .iter()
                        .map(|operand| self.replace_free(name, value, operand.clone()))
                        .collect::<Vec<Expr>>(),
                },
            },
            Expr::Var { name: v_name } if v_name == name => value.clone(),
            _ => ast,
        };

        new_expr
    }

    fn eval_var(&mut self, ast: &Expr) -> Result<Expr, String> {
        match ast {
            Expr::Var { name } => {
                let value = self.lookup(&name);

                match value {
                    Some(expr) => {
                        let expr = expr.clone();
                        self.eval(&expr)
                    }
                    None => Err(format!("Variable '{}' is not defined", name)),
                }
            }
            ast => Err(format!("eval_var cannot evaluate '{:?}'", ast)),
        }
    }

    fn eval_literal(&mut self, ast: &Expr) -> Result<Expr, String> {
        match ast {
            Expr::Literal(lit) => Ok(Expr::Literal(lit.clone())),
            ast => Err(format!("eval_literal cannot evaluate '{:?}'", ast)),
        }
    }

    fn eval_expr_keyword(&mut self, ast: &Expr) -> Result<Expr, String> {
        match ast {
            Expr::Expr { operator, operands } => match *operator.clone() {
                Expr::Keyword(kw) => match kw {
                    Keyword::Def => match operands.len() {
                        3 => match operands.get(0).unwrap() {
                            Expr::Var { name } => {
                                let value = self.eval(operands.get(1).unwrap())?;

                                self.stack.push_back(Frame::new(name.clone(), value.clone()));
                                let result =
                                    self.eval(
                                        &self.replace_free(
                                            name,
                                            &value,
                                            operands.get(2).unwrap().clone()
                                        )
                                    );

                                self.stack.pop_back();

                                result
                            }
                            e => Err(format!("invalid variable name '{:?}'", e)),
                        },
                        l if l > 3 => match operands.get(0).unwrap() {
                            Expr::Var { name } => {
                                let value = self.eval(operands.get(1).unwrap())?;

                                self.push_var(name.to_owned(), value.clone());

                                let new_operator = self.replace_free(name, &value, operands.get(2).unwrap().clone());
                                let new_operands = operands[3..].to_vec().into_iter().map(|op| self.replace_free(name, &value, op)).collect::<Vec<Expr>>();

                                self.eval(&Expr::Expr { operator: Box::new(new_operator), operands: new_operands })
                            }
                            e => Err(format!("invalid variable name '{:?}'", e)),
                        },
                        l => Err(format!(
                            "def (δ) expected 3 arguments. {} arguments were provided. Example usage: (δ x 5 (ι x))",
                            l
                        )),
                    },
                    Keyword::Lambda => match operands.len() {
                      2 => Ok(Expr::Expr { operator: Box::new(Expr::Keyword(Keyword::Lambda)), operands: operands.clone() }),
                      3 => {
                        match operands.get(0).unwrap() {
                          Expr::Var { name } => {
                            let value = self.eval(operands.get(2).unwrap()).unwrap();
                            self.stack.push_back(Frame::new(name.to_owned(), value.clone()));

                            let result = self.eval(&self.replace_free(name, &value, operands.get(1).unwrap().clone()));

                            self.stack.pop_back();

                            result
                          },
                          Expr::Literal(expected) => match self.eval(operands.get(2).unwrap()).unwrap() {
                            Expr::Literal(actual) if actual == *expected => self.eval(operands.get(1).unwrap()),
                            actual => Err(format!("lambda (λ) expected '{:?}' but received '{:?}'", expected, actual))
                          },
                          Expr::Keyword(Keyword::Ignore) => self.eval(operands.get(1).unwrap()),
                          id => Err(format!("Invalid lambda (λ) argument: {:?}", id))
                      }
                      },
                      4.. => {
                        match operands.get(0).unwrap() {
                          Expr::Var { name } => {
                            let value = self.eval(operands.get(2).unwrap()).unwrap();
                            self.stack.push_back(Frame::new(name.to_owned(), value));

                            let new_operator = self.eval(operands.get(1).unwrap()).unwrap();

                            let result = self.eval(&Expr::Expr { operator: Box::new(new_operator), operands: operands.get(3..).unwrap()[..].to_vec() });

                            self.stack.pop_back();

                            result
                          },
                          Expr::Literal(expected) => {match self.eval(operands.get(2).unwrap()).unwrap() {
                            Expr::Literal(actual) if actual == *expected => {
                              let new_op = self.eval(operands.get(1).unwrap()).unwrap();

                              self.eval(&Expr::Expr { operator: Box::new(new_op), operands: operands[3..].to_vec() })
                            },
                            actual => Err(format!("lambda (λ) expected '{}' but received '{}'", expected, actual))
                          }},
                          Expr::Keyword(Keyword::Ignore) => {
                            let new_op = self.eval(operands.get(1).unwrap()).unwrap();
                            self.eval(&Expr::Expr { operator: Box::new(new_op), operands: operands[3..].to_vec() })
                          },
                          id => Err(format!("Invalid lambda (λ) argument: {:?}", id))
                      }
                      },
                      l => Err(format!("lambda (λ) expected 2 or more arguments but {} arguments were provided.", l))
                    },
                    Keyword::Arrow => unimplemented!(),
                    Keyword::External => unimplemented!(),
                    Keyword::Id =>
                      match operands.len() {
                        0 => Ok(Expr::Expr { operator: Box::new(Expr::Keyword(Keyword::Id)), operands: vec![] }),
                        1 => {
                          let expr = operands.get(0).unwrap();
                          self.eval(expr)
                        },
                        _ => {
                          let new_operator = {
                            let operator_expr = operands.get(0).unwrap();

                            self.eval(operator_expr).unwrap()
                          };

                          let new_operands = operands.get(1..).unwrap()[..].to_vec();

                          self.eval(&Expr::Expr { operator: Box::new(new_operator), operands: new_operands })
                        }
                      },
                    Keyword::Ignore => match operands.len() {
                      0 => Ok(Expr::Keyword(Keyword::Ignore)),
                      1 => Ok(Expr::Keyword(Keyword::Nil)),
                      2 => Ok(operands.get(2).unwrap().clone()),
                      _ => {
                        let new_operator = {
                          let operator_expr = operands.get(1).unwrap();
                          self.eval(operator_expr).unwrap()
                        };

                        let new_operands = operands.get(2..).unwrap()[..].to_vec();

                        self.eval(&Expr::Expr { operator: Box::new(new_operator), operands: new_operands })
                      }
                    },
                    Keyword::Nil => Ok(Expr::Keyword(Keyword::Nil)),
                },
                op => Err(format!(
                    "eval_expr_keyword cannot evaluate an operator '{}', {}:{}",
                    op,
                    file!(),
                    line!()
                )),
            },
            ast => Err(format!("eval_expr_keyword cannot evaluate '{}'", ast)),
        }
    }

    fn eval_expr_nested(&mut self, ast: &Expr) -> Result<Expr, String> {
        match ast {
            Expr::Expr { operator, operands } => match *operator.clone() {
                Expr::Expr {
                    operator: inner_op,
                    operands: inner_operands,
                } => {
                    let mut new_operands = vec![];
                    new_operands.extend(inner_operands.into_iter());
                    new_operands.extend(operands.into_iter().map(|x| x.clone()));

                    let new = Expr::Expr {
                        operator: inner_op,
                        operands: new_operands,
                    };

                    self.eval(&new)
                }
                op => Err(format!(
                    "eval_expr_nested cannot evaluate an operator '{}'",
                    op
                )),
            },
            ast => Err(format!(
                "eval_expr_nested cannot evaluate an expression '{}'",
                ast
            )),
        }
    }

    fn eval_expr_var(&mut self, ast: &Expr) -> Result<Expr, String> {
        match ast {
            Expr::Expr { operator, operands } => match *operator.clone() {
                Expr::Var { name } => {
                    let new_operator = self.eval(&Expr::Var { name }).unwrap();

                    match new_operator {
                        Expr::Expr {
                            operator: new_operator,
                            operands: var_operands,
                        } => {
                            let mut new_operands = vec![];

                            new_operands.extend(var_operands.into_iter());
                            new_operands.extend(operands.clone());

                            self.eval(&Expr::Expr {
                                operator: new_operator,
                                operands: new_operands,
                            })
                        }
                        new_operator => self.eval(&Expr::Expr {
                            operator: Box::new(new_operator),
                            operands: operands.clone(),
                        }),
                    }
                }
                op => Err(format!(
                    "eval_expr_var cannot evaluate an operator '{:?}', {}:{}",
                    op,
                    file!(),
                    line!()
                )),
            },
            ast => Err(format!("eval_expr_var cannot evaluate '{:?}'", ast)),
        }
    }

    pub fn eval(&mut self, ast: &Expr) -> Result<Expr, String> {
        self.eval_var(ast)
            .or_else(|_| self.eval_literal(ast))
            .or_else(|_| self.eval_expr_keyword(ast))
            .or_else(|_| self.eval_expr_nested(ast))
            .or_else(|_| self.eval_expr_var(ast))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        evaluator::Runtime,
        keywords::Keyword,
        lexer::Lexer,
        literal::Literal,
        parse,
        parser::{run_parser, Expr},
    };

    macro_rules! t {
        ($src:expr, $name:expr, $val:expr, $expected:expr) => {
            assert_eq!(
                format!(
                    "{}",
                    Runtime::new().replace_free(&$name.to_string(), &parse!($val), parse!($src))
                ),
                $expected
            );
        };
    }

    #[test]
    fn replace_free_lambda() {
        t!("(x (λ x x) x)", "x", "1", "(1 (λ x x) 1)");
        t!(
            "(f (λ f (f x)) 5)",
            "f",
            "(λ _ 5)",
            "((λ _ 5) (λ f (f x)) 5)"
        );
    }

    #[test]
    fn replace_free_def() {
        t!("(x (δ x x) y x)", "x", "1", "(1 (δ x x) y 1)");
        t!("(δ f (λ x y x))", "y", "ι", "(δ f (λ x ι x))");
        t!(
            "(δ true (λ p (λ q p))
      (δ false (λ p (λ q q))
        true
      )
    )",
            "p",
            "42",
            "(δ true (λ p (λ q p)) (δ false (λ p (λ q q)) true))"
        );
    }
}
