/**
 * The derive macro implementation that implements the trait 'Lexer' for a
 * token-type enum.
 */

extern crate proc_macro;
extern crate yk_dense_fsa;
extern crate syn;
extern crate quote;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemEnum, LitStr, Fields, Ident};
use quote::quote;
use yk_dense_fsa::{nfa, dfa};
use yk_dense_fsa::yk_regex_parse as regex;
use yk_dense_fsa::yk_intervals::{LowerBound, UpperBound};

// Identifier for the front-end lexer library
const FRONT_LIBRARY_NAME: &str = "yk_lexer";
// Regular expression for a C-style identifier
const C_IDENT_REGEX: &str = "[A-Za-z_][A-Za-z0-9_]*";
// Attribute name for error
const ATTRIBUTE_ERR: &str = "error";
// Attribute name for end
const ATTRIBUTE_END: &str = "end";
// Attribute name for a C-style identifier
const ATTRIBUTE_C_IDENT: &str = "c_ident";
// Attribute name for a regex-token
const ATTRIBUTE_REGEX: &str = "regex";
// Attribute name for a raw-string token
const ATTRIBUTE_TOKEN: &str = "token";

struct TokenDefinition {
    variant_ident: Ident,
    regex_str: String,
    precedence: usize,
}

struct LexerData {
    enum_name: Ident,
    err_variant: Ident,
    end_variant: Ident,
    tokens: Vec<TokenDefinition>,
}

#[proc_macro_derive(Lexer, attributes(
    error,
    end,
    c_ident,
    regex,
    token,
))]
pub fn yk_lexer(item: TokenStream) -> TokenStream {
    // Identifier for the front-end lexer library
    let FRONT_LIBRARY = quote::format_ident!("{}", FRONT_LIBRARY_NAME);

    // Parse the enum
    let enm = parse_macro_input!(item as ItemEnum);
    let lexer_data = parse_attributes(&enm);

    let enum_name = lexer_data.enum_name;
    let error_token = lexer_data.err_variant;

    // Now we have the regexes, let's construct a DFA
    let mut nfa = nfa::Automaton::new();
    for TokenDefinition{ variant_ident, regex_str, precedence } in lexer_data.tokens {
        let regex_ast = regex::parse(&regex_str).expect("Error in regex syntax!"); // TODO: Good error msg
        nfa.add_regex_with_accepting_value(&regex_ast, (variant_ident, precedence));
    }

    // Determinize the state machine
    let dfa = dfa::Automaton::from_nfa(nfa, |a, b| {
        if a.1 > b.1 {
            a
        }
        else if a.1 < b.1 {
            b
        }
        else {
            panic!("{} and {} are conflicting!", a.0, b.0);
        }
    });

    // We collect each arm of the match
    let mut state_transitions = Vec::new();
    for state in dfa.states() {
        let mut arms = Vec::new();

        // We visit the state's possible transitions
        if let Some(transitions) = dfa.transitions_from(&state) {
            for (interval, destination) in transitions {
                // We need to generate an arm
                let lower = to_lower_inclusive_u32(&interval.lower);
                let upper = to_upper_inclusive_u32(&interval.upper);

                let arm_pattern = match (lower, upper) {
                    (Some(a), Some(b)) => quote!{ #a..=#b },
                    (Some(a), None) => quote!{ #a.. },
                    (None, Some(b)) => quote!{ ..=#b },
                    (None, None) => quote!{ .. },
                };

                // Build a "save" statement if the state is an accepting one
                let acceptor = match dfa.accepting_value(&destination) {
                    Some((token_type, _)) => quote!{
                        last_accepting = Some((current_index, #enum_name::#token_type))
                    },
                    None => quote!{},
                };

                // Build the actual match arm
                let destination_id = destination.id();
                let arm = quote!{
                    #arm_pattern => {
                        current_state = #destination_id;
                        #acceptor;
                    },
                };

                arms.push(arm);
            }
        }

        // Add a default failing arm
        arms.push(quote!{
            _ => {
                if let Some((idx, value)) = last_accepting {
                    // We succeeded before, return that
                    return (idx, value);
                }
                else {
                    // No success before, return an error
                    return (1, #enum_name::#error_token);
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

    // Wrap it into an internal token parsing function
    let initial_state_id = dfa.start.id();
    let end_token = lexer_data.end_variant;
    let res = quote!{
        impl ::#FRONT_LIBRARY::LexerInternal<#enum_name> for #enum_name {
            fn next_token_internal(source: &str) -> (usize, #enum_name) {
                let mut source_it = source.char_indices();
                let mut current_state = #initial_state_id;
                let mut last_accepting = None;
                let mut has_consumed = false;
                loop {
                    if let Some((current_index, current_char)) = source_it.next() {
                        has_consumed = true;
                        match current_state {
                            #(#state_transitions)*
                        }
                    }
                    else {
                        if let Some((idx, value)) = last_accepting {
                            // We succeeded before, return that
                            return (idx, value);
                        }
                        else if has_consumed {
                            // No success, but there are characters consumed, we error out
                            return (1, #enum_name::#error_token);
                        }
                        else {
                            // Nothing consumed, no more characters, it's just the end on input
                            return (0, #enum_name::#end_token);
                        }
                    }
                }
            }
        }

        impl ::#FRONT_LIBRARY::TokenType<#enum_name> for #enum_name {
            fn with_source(source: &str) -> ::#FRONT_LIBRARY::BuiltinLexer<#enum_name> {
                ::#FRONT_LIBRARY::BuiltinLexer::with_source(source)
            }
        }
    };
    res.into()
}

fn parse_attributes(enm: &ItemEnum) -> LexerData {
    // We need to fill these
    let enum_name = enm.ident.clone();
    let mut end_variant = None;
    let mut err_variant = None;
    let mut tokens = Vec::new();

    // Parse the variants
    for variant in &enm.variants {
        // If it's not a basic enum variant (without fields), we error out
        match variant.fields {
            Fields::Unit => { },
            _ => panic!("Token types can only be unit-like variants!"),
        }

        let variant_ident = variant.ident.clone();

        for attr in &variant.attrs {
            if attr.path.is_ident(ATTRIBUTE_END) {
                assert!(end_variant.is_none(), "Exactly on 'end' variant must be defined!");
                end_variant = Some(variant_ident.clone());
            }
            else if attr.path.is_ident(ATTRIBUTE_ERR) {
                assert!(end_variant.is_none(), "Exactly on 'err' variant must be defined!");
                err_variant = Some(variant_ident.clone());
            }
            else if attr.path.is_ident(ATTRIBUTE_TOKEN) {
                // TODO: Allow '=' too
                let token = attr.parse_args::<LitStr>().unwrap();
                // TODO: Escape so it actually wouldn't be a regex
                let regex_str = token.value();
                tokens.push(TokenDefinition{
                    variant_ident: variant_ident.clone(),
                    regex_str,
                    precedence: 1,
                });
            }
            else if attr.path.is_ident(ATTRIBUTE_C_IDENT) {
                assert!(attr.tokens.is_empty(), "'c_ident' requires no arguments!");
                tokens.push(TokenDefinition{
                    variant_ident: variant_ident.clone(),
                    regex_str: C_IDENT_REGEX.into(),
                    precedence: 0,
                });
            }
            else if attr.path.is_ident(ATTRIBUTE_REGEX) {
                // TODO: Allow '=' too
                let token = attr.parse_args::<LitStr>().unwrap();
                let regex_str = token.value();
                tokens.push(TokenDefinition{
                    variant_ident: variant_ident.clone(),
                    regex_str,
                    precedence: 0,
                });
            }
        }
    }

    let err_variant = err_variant.expect("An 'error' variant must be defined!");
    let end_variant = end_variant.expect("An 'end' variant must be defined!");

    LexerData{ enum_name, err_variant, end_variant, tokens, }
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
