use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    Def,      // 2
    Lambda,   // 2
    Arrow,    // 1+
    External, // 2
    Id,       // 1
    Ignore,   // 0
    Nil,      // 0
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::Def => f.write_str("δ"),
            Keyword::Lambda => f.write_str("λ"),
            Keyword::Arrow => f.write_str("->"),
            Keyword::External => f.write_str("ε"),
            Keyword::Id => f.write_str("ι"),
            Keyword::Ignore => f.write_str("_"),
            Keyword::Nil => f.write_str("Ω"),
        }
    }
}
