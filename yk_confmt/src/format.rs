/**
 * Structures related to formatting (color, style, ...).
 */

pub struct Fmt {
    pub fg: Fg,
    pub bg: Bg,
}

pub enum Col {
    White,
    Red,
    Green,
    Blue,
}

pub struct Fg(Col);
pub struct Bg(Col);
