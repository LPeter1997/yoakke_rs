/**
 * The main lexer library that connects with the derive library for the
 * derive-macro.
 */

extern crate yk_lexer_derive;

mod position;
mod lexer;
mod token;

pub use yk_lexer_derive::Lexer;

pub use position::Position;
pub use lexer::{LexerState, Lexer, StandardLexer, Modification};
pub use token::{TokenType, Token};
