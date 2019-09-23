
extern crate yk_lexer;

#[derive(yk_lexer::Lexer)]
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