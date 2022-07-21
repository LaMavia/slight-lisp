use std::{collections::HashMap, ops::Index};

use crate::parser::Expr;

#[derive(Debug)]
pub struct Frame {
    variables: HashMap<String, Expr>,
}

impl Frame {
    pub fn new(name: String, value: Expr) -> Self {
        let mut s = Self {
            variables: HashMap::new(),
        };

        s.variables.insert(name, value);

        s
    }

    pub fn lookup(&self, name: &String) -> Option<&Expr> {
        self.variables.get(name)
    }

    pub fn push(&mut self, name: String, value: Expr) {
        self.variables.insert(name, value);
    }
}
