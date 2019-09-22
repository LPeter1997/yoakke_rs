
extern crate yk_lexer_derive;

#[derive(yk_lexer_derive::Lexer)]
enum TokenType {
    #[error]
    Error,

    #[end]
    End,

    #[regex("[asd]")]
    Foo,

    #[regex("[bte]")]
    Bar,
}

fn main() {
    let tt = TokenType::End;
}
