use std::vec;

use crate::{keywords::Keyword, literal::Literal, position::Position};

#[derive(Debug, Clone, PartialEq)]
pub enum Lexem {
    Keyword(Keyword),
    Literal(Literal),
    ParenthesisOpen,
    ParenthesisClose,
    Identifier(String),
}

pub type Token = (Lexem, Position);

pub struct Lexer {
    current: String,
    position: Position,
    in_string: bool,
    pub tokens: Vec<Token>,
}

impl Lexer {
    pub fn new() -> Self {
        Self {
            current: "".to_string(),
            position: Position::new(0, 0),
            tokens: vec![],
            in_string: false,
        }
    }

    fn sublex_keyword(&self) -> Option<Token> {
        let position = self.position.clone();

        match self.current.as_str() {
            "δ" | "def" => Some((Lexem::Keyword(Keyword::Def), position)),
            "ε" | "external" => Some((Lexem::Keyword(Keyword::External), position)),
            "λ" | "lambda" => Some((Lexem::Keyword(Keyword::Lambda), position)),
            "->" => Some((Lexem::Keyword(Keyword::Arrow), position)),
            "ι" | "id" => Some((Lexem::Keyword(Keyword::Id), position)),
            "_" => Some((Lexem::Keyword(Keyword::Ignore), position)),
            "Ω" | "nih" => Some((Lexem::Keyword(Keyword::Nil), position)),
            _ => None,
        }
    }

    fn sublex_literal(&self) -> Option<Token> {
        Literal::num(&self.current)
            .or(Literal::string(&self.current))
            .or(Literal::nil(&self.current))
            .map(|l| (Lexem::Literal(l), self.position.clone()))
    }

    fn sublex_identifier(&self) -> Option<Token> {
        if self.current.is_empty() || self.current.contains("\"") {
            None
        } else {
            Some((
                Lexem::Identifier(self.current.clone()),
                self.position.clone(),
            ))
        }
    }

    fn push(&mut self, token: Option<Lexem>) -> Result<(), String> {
        let result = match self
            .sublex_keyword()
            .or_else(|| self.sublex_literal())
            .or_else(|| self.sublex_identifier())
            .map(|token| self.tokens.push(token))
        {
            Some(_) => Ok(()),
            None if self.current.is_empty() => Ok(()),
            None => Err(format!("Cannot parse {}", self.current)),
        };

        self.current = "".to_string();

        if let Some(t) = token {
            if !self.current.is_empty() {
                self.position.next_col();
            }

            self.tokens.push((t, self.position.clone()))
        }

        result
    }

    pub fn lex(&mut self, source: &String) -> Result<(), String> {
        for c in source.chars() {
            self.position.next_col();

            if c.is_whitespace() && !self.in_string {
                self.push(None).unwrap();

                if c == '\n' {
                    self.position.next_row();
                } else {
                    self.position.next_col();
                }
            } else {
                match c {
                    '(' => self.push(Some(Lexem::ParenthesisOpen)).unwrap(),
                    ')' => self.push(Some(Lexem::ParenthesisClose)).unwrap(),
                    c => {
                        if c == '"' {
                            self.in_string = !self.in_string;
                        }

                        self.current.push(c)
                    }
                };
            }
        }

        self.push(None)?;

        Ok(())
    }

    pub fn lexems(&self) -> Vec<&Token> {
        self.tokens.iter().map(|t| t).collect::<Vec<&Token>>()
    }
}
