#[derive(Debug, Clone, Copy)]
pub struct Position {
    row: usize,
    col: usize,
}

impl Position {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub fn next_col(&mut self) -> Self {
        self.col += 1;

        *self
    }

    pub fn next_row(&mut self) -> Self {
        self.col = 0;
        self.row += 1;

        *self
    }
}
