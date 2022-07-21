use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Num(f64),
    String(String),
    Nil,
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Num(num) => f.write_fmt(format_args!("{}", num)),
            Literal::String(string) => f.write_fmt(format_args!("'{}'", string)),
            Literal::Nil => f.write_str("Φ"),
        }
    }
}

impl Literal {
    pub fn num(s: &String) -> Option<Literal> {
        match s.parse::<f64>() {
            Ok(f) => Some(Literal::Num(f)),
            Err(_) => None,
        }
    }

    pub fn string(s: &String) -> Option<Literal> {
        if s.len() <= 1 {
            return None;
        }

        let has_opening_quote = s.chars().next().unwrap() == '"';
        let has_closing_quote = s.chars().last().unwrap() == '"';

        if !(has_opening_quote && has_closing_quote) {
            return None;
        }

        let mut stream = s.chars().peekable().enumerate();

        while let Some((i, c)) = stream.next() {
            if c == '"' && !(i == 0 || i == s.len() - 1) {
                return None;
            }

            if c == '\\' && i == s.len() - 1 {
                return None;
            }

            if c == '\\' {
                stream.next();
            }
        }

        return Some(Literal::String(s[1..=(s.len() - 2)].to_string()));
    }

    pub fn nil(s: &String) -> Option<Literal> {
        if s.as_str() == "Φ" || s.as_str() == "nil" {
            Some(Literal::Nil)
        } else {
            None
        }
    }
}
