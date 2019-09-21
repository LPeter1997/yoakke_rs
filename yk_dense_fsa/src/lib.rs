
extern crate yk_intervals;
extern crate yk_regex_parse;

pub mod nfa;
pub mod dfa;

#[test]
fn foo() {
    let rx = r"[A-Za-z_][A-Za-z_0-9]*";
    let ast = yk_regex_parse::parse(rx).unwrap();
    println!("AST: {:?}", ast);
    let nf = nfa::Automaton::from(ast);
    println!("NFA: {:?}", nf);
    let df = dfa::Automaton::from(nf);
    println!("DFA: {:?}", df);
}
