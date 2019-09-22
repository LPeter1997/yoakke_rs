
extern crate proc_macro;
extern crate yk_dense_fsa;
extern crate syn;

use std::collections::HashMap;
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemEnum, LitStr, Fields};
use yk_dense_fsa::{nfa, dfa};
use yk_dense_fsa::yk_regex_parse as regex;

struct Token<'a, T> {
    pub kind: T,
    pub value: &'a str,
}

const C_IDENT_REGEX: &str = "[A-Za-z_][A-Za-z0-9_]*";

#[proc_macro_derive(Lexer, attributes(
    error,
    end,
    c_ident,
    regex,
))]
pub fn yk_lexer(item: TokenStream) -> TokenStream {
    // Parse the enum
    let enm = parse_macro_input!(item as ItemEnum);

    // Things we need to fill
    let mut err_variant = None;
    let mut end_variant = None;

    let mut regexes = Vec::new();

    for variant in &enm.variants {
        match variant.fields {
            Fields::Unit => { },
            _ => panic!("Tokens can't hold extra information!"),
        }

        let var_attrs = &variant.attrs;

        for attr in var_attrs {
            let ident = &attr.path.segments[0].ident;

            if attr.path.segments.len() != 1 {
                panic!("Unknown attribute!");
            }

            if ident == "error" {
                if err_variant.is_some() {
                    panic!("You can only define one 'error' variant!");
                }
                if !attr.tokens.is_empty() {
                    panic!("'error' can't have any metadata!");
                }

                err_variant = Some(ident.clone());
            }
            else if ident == "end" {
                if end_variant.is_some() {
                    panic!("You can only define one 'end' variant!");
                }
                if !attr.tokens.is_empty() {
                    panic!("'end' can't have any metadata!");
                }

                end_variant = Some(ident.clone());
            }
            else if ident == "c_ident" {
                if !attr.tokens.is_empty() {
                    panic!("'c_ident' can't have any metadata!");
                }

                regexes.push((ident.clone(), String::from(C_IDENT_REGEX)));
            }
            else if ident == "regex" {
                // TODO: Allow '=' too
                let rx = attr.parse_args::<LitStr>().unwrap();
                let rx_str = rx.value();

                regexes.push((ident.clone(), rx_str));
            }
            else {
                panic!("Unknown attribute!");
            }
        }
    }

    if err_variant.is_none() {
        panic!("You must define an 'error' variant!");
    }
    if end_variant.is_none() {
        panic!("You must define an 'end' variant!");
    }

    // Now we have the regexes, let's construct a DFA
    let mut nfa = nfa::Automaton::new();
    for (variant, rx) in regexes {
        let regex_ast = regex::parse(&rx).unwrap(); // TODO: Good error msg
        nfa.add_regex_with_accepting_value(&regex_ast, variant);
    }

    // Determinize the state machine
    let dfa = dfa::Automaton::from_nfa(nfa, |_, _| panic!("Multiple accepting values!"));

    // Construct the finite automaton
    /*
     * It would look something like this:
     *
     * // src: &str
     *
     * let mut src_it = src.char_indicies();
     * let mut state = initial_state;
     * let mut last_accepting = None;
     * loop {
     *     if let Some(idx, c) = char_indicies.next() {
     *         match state {
     *             State(some_state) => match c as u32 {
     *                 ('a' as u32)..=('z' as u32) => {
     *                     state = sone_next_state;
     *                     // If accepting state is some accepting state
     *                     last_accepting = Some((idx, state));
     *                 },
     *                 _ => if last_accepting.is_some() { Ok! } else { Error! }
     *             },
     *             // Other states
     *         }
     *     }
     *     else {
     *         // Return last accepting, then return end
     *     }
     * }
     */

    // We collect each arm of the match
    //let mut state_transitions = Vec::new();
    for state in dfa.states() {
        // We visit the state's possible transitions
        if let Some(transitions) = dfa.transitions_from(&state) {
            for (interval, destination) in transitions {
                // We need to generate an arm
                /*
                 * (interval.lower to interval.upper) => {
                 *     Change to the destination state
                 *     If destination state is accepting, save
                 * }
                 */
                /*let arm = quote!{

                };*/
            }
        }
        // Add a default failing arm
    }

    TokenStream::new()
}
