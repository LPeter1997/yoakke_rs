
extern crate yk_lexer;
extern crate rand;
extern crate rand_pcg;

mod rnd;
mod str_gen;
mod fuzz_gen;

use yk_lexer::{StandardLexer, Lexer, TokenType, Token};
use str_gen::*;
use fuzz_gen::*;
use std::io::Write;

#[derive(Lexer, Clone, PartialEq, Eq, Debug)]
enum TokenKind {
    #[error]
    Error,

    #[end]
    End,

    #[regex(r"[ \n]")]
    #[ignore]
    Ws,

    #[c_ident]
    Ident,

    #[regex(r"[0-9]+")]
    IntLit,

    #[token("(")]
    LP,

    #[token(")")]
    RP,

    #[token("if")]
    KwIf,

    #[token("else")]
    KwElse,
}

const charset: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ()0123456789 \n";

fn main() {
    let seed = if true {
        rnd::seed_from_system_time()
    }
    else {
        1569947473
    };
    rnd::set_seed(seed);
    println!("Seed: {}", rnd::current_seed());

    let fs = create_fuzz_strategy();
    fuzz(10, 100, &fs);
}

fn create_fuzz_strategy() -> RandomEdit {
    let mut fs = RandomEdit::new();
    fs.add(AppendEdit::with_gen(create_string_strategy()));
    fs.add(InsertEdit::with_gen(create_string_strategy()));
    fs.add(EraseEdit::new());
    fs.add(SpliceEdit::with_gen(create_string_strategy()));
    fs
}

fn create_string_strategy() -> RandomStringStrategy {
    let mut sg = RandomStringStrategy::new();
    sg.add(RandomStringGenerator::with_len_and_charset(0..25, charset));
    {
        let mut tg = RandomTokenGenerator::new();
        tg.add("if");
        tg.add("else");
        tg.add("(");
        tg.add(")");
        sg.add(tg);
    }
    sg
}

fn fuzz(epochs: usize, edits_per_epoch: usize, strat: &dyn FuzzStrategy) {
    for _ in 0..epochs {
        fuzz_epoch(edits_per_epoch, strat);
    }
}

fn fuzz_epoch(edits: usize, strat: &dyn FuzzStrategy) {
    let mut lexer = TokenKind::lexer();
    let mut tokens = Vec::new();

    for _ in 0..edits {
        let orig_source: String = lexer.source().into();
        let prev_tokens: Vec<_> = lexer.iter().collect();
        let (erased, inserted) = strat.make_edit(lexer.source());
        let m = lexer.modify(&tokens, erased.clone(), &inserted);
        let m = m.apply(&mut tokens);

        let erased_cnt = m.erased.len();
        let inserted_cnt = m.inserted;

        let orig_tokens: Vec<_> = lexer.iter().collect();
        let diff = tokens.len() - inserted_cnt;
        println!("tokens: {}, erased: {}, inserted: {} (saved: {})", tokens.len(), erased_cnt, inserted_cnt, diff);

        if !eq_ignore_pos(&tokens, &orig_tokens) {
            println!("While editing source: '{}'", orig_source);
            println!("erase: {:?}, insert : '{}'", erased, inserted);
            println!("That became         : '{}'\n", lexer.source());

            print!("Prev    : [");
            for t in &prev_tokens {
                print!("\"{}\", ", &orig_source[t.range.clone()]);
            }
            println!("]");
            print!("Expected: [");
            for t in &orig_tokens {
                print!("\"{}\", ", &lexer.source()[t.range.clone()]);
            }
            println!("]");

            print!("Got     : [");
            for t in &tokens {
                print!("\"{}\", ", &lexer.source()[t.range.clone()]);
            }
            println!("]\n");

            println!("===========================================\n");
            println!("Details:\n");
            println!("erase range: {:?}\n", m.erased);
            print!("Expected: [");
            for t in &orig_tokens {
                print!("{:?}, ", t);
            }
            println!("]\n");

            print!("Got     : [");
            for t in &tokens {
                print!("{:?}, ", t);
            }
            println!("]\n");

            println!("Seed: {}", rnd::current_seed());

            std::io::stdout().flush();

            panic!("Incremental mismatch!");
        }
    }
}

// TODO: We are ignoring position but we shouldn't!
fn eq_ignore_pos(v1: &Vec<Token<TokenKind>>, v2: &Vec<Token<TokenKind>>) -> bool {
    if v1.len() != v2.len() {
        false
    }
    else {
        for i in 0..v1.len() {
            let t1 = &v1[i];
            let t2 = &v2[i];
            if t1.range != t2.range || t1.kind != t2.kind || t1.lookahead != t2.lookahead {
                return false;
            }
        }
        return true;
    }
}
