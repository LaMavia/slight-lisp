use std::error::Error;

use evaluator::Runtime;
use lexer::Lexer;
use parser::{run_parser, Expr};

use crate::literal::Literal;

mod evaluator;
mod frame;
mod keywords;
mod lexer;
mod literal;
mod parser;
mod position;
mod repl;

fn main() -> Result<(), Box<dyn Error>> {
    repl::run();

    Ok(())
}
