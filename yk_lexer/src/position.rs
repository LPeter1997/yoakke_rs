/**
 * Position representation in a file.
 */

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub column: usize,
    pub line: usize,
}

impl Position {
    pub fn new() -> Position {
        Position{ column: 0, line: 0 }
    }

    pub fn newline(&mut self) {
        self.column = 0;
        self.line += 1;
    }

    pub fn advance_columns(&mut self, count: usize) {
        self.column += count;
    }
}
