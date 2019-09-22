
extern crate proc_macro;
extern crate yk_dense_fsa;
extern crate syn;
extern crate quote;

use std::collections::HashMap;
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemEnum, LitStr, Fields};
use quote::quote;
use yk_dense_fsa::{nfa, dfa};
use yk_dense_fsa::yk_regex_parse as regex;
use yk_dense_fsa::yk_intervals::{Interval, LowerBound, UpperBound};

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

                regexes.push((variant.ident.clone(), String::from(C_IDENT_REGEX)));
            }
            else if ident == "regex" {
                // TODO: Allow '=' too
                let rx = attr.parse_args::<LitStr>().unwrap();
                let rx_str = rx.value();

                regexes.push((variant.ident.clone(), rx_str));
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

    let err_variant = err_variant.unwrap();
    let end_variant = end_variant.unwrap();
    let enum_name = enm.ident.clone();

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
     *     if let Some(idx, c) = src_it.next() {
     *         match state {
     *             State(some_state) => match c as u32 {
     *                 ('a' as u32)..=('z' as u32) => {
     *                     state = sone_next_state;
     *                     // If accepting state is some accepting state
     *                     last_accepting = Some((idx, value));
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
    let mut state_transitions = Vec::new();
    for state in dfa.states() {
        let mut arms = Vec::new();

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
                let lower = to_lower_inclusive_u32(&interval.lower);
                let upper = to_upper_inclusive_u32(&interval.upper);

                let arm_case =
                    if lower.is_some() && upper.is_some() {
                        let lower = lower.unwrap();
                        let upper = upper.unwrap();
                        quote!{ #lower..=#upper }
                    }
                    else if lower.is_some() {
                        let lower = lower.unwrap();
                        quote!{ #lower.. }
                    }
                    else if upper.is_some() {
                        let upper = upper.unwrap();
                        quote!{ ..#upper }
                    }
                    else {
                        quote!{ .. }
                    };

                let destination_id = destination.id();
                let arm = if let Some(accepting) = dfa.accepting_value(&destination) {
                    // Accepting state
                    quote!{
                        #arm_case => {
                            current_state = #destination_id;
                            last_accepting = Some((current_index, #enum_name::#accepting));
                        },
                    }
                }
                else {
                    // Non-accepting state
                    quote!{
                        #arm_case => {
                            current_state = #destination_id;
                        },
                    }
                };
                arms.push(arm);
            }
        }

        // Add a default failing arm
        arms.push(quote!{
            _ => {
                if let Some((idx, value)) = last_accepting {
                    return Some((&source[0..idx], value));
                }
            },
        });

        // Add the arms to all the state arms
        let state_id = state.id();
        state_transitions.push(quote!{
            #state_id => match current_char as u32 {
                #(#arms)*
            },
        });
    }
    // Add a default to the transitions
    state_transitions.push(quote!{
        _ => panic!("Unknown state!"),
    });
    // Wrap it in the whole logic
    let initial_state_id = dfa.start.id();
    let res = quote!{
        fn lex(source: &str) -> Option<(&str, #enum_name)> {
            let mut source_it = source.char_indices();
            let mut current_state = #initial_state_id;
            let mut last_accepting = None;
            loop {
                if let Some((current_index, current_char)) = source_it.next() {
                    match current_state {
                        #(#state_transitions)*
                    }
                }
                else {
                    if let Some((idx, value)) = last_accepting {
                        return Some((&source[0..idx], value));
                    }
                    else {
                        unimplemented!();
                    }
                }
            }
        }
    };
    res.into()
}

fn to_lower_inclusive_u32(b: &LowerBound<char>) -> Option<u32> {
    match b {
        LowerBound::Excluded(c) => Some(*c as u32 + 1),
        LowerBound::Included(c) => Some(*c as u32),
        LowerBound::Unbounded => None,
    }
}

fn to_upper_inclusive_u32(b: &UpperBound<char>) -> Option<u32> {
    match b {
        UpperBound::Excluded(c) => Some(*c as u32 - 1),
        UpperBound::Included(c) => Some(*c as u32),
        UpperBound::Unbounded => None,
    }
}
