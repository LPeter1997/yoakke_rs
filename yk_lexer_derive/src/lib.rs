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
// Attribute to ignore the defined token
const ATTRIBUTE_IGNORE: &str = "ignore";

struct TokenDefinition {
    variant_ident: Ident,
    regex_str: String,
    precedence: usize,
    ignore: bool,
}

struct LexerData {
    enum_name: Ident,
    err_variant: Ident,
    end_variant: Ident,
    tokens: Vec<TokenDefinition>,
}

#[derive(Clone)]
struct AcceptingState {
    variant_ident: Ident,
    precedence: usize,
    ignore: bool,
}

#[proc_macro_derive(Lexer, attributes(
    error,
    end,
    c_ident,
    regex,
    token,
    ignore,
))]
pub fn yk_lexer(item: TokenStream) -> TokenStream {
    // Identifier for the front-end lexer library
    let FRONT_LIBRARY = quote::format_ident!("{}", FRONT_LIBRARY_NAME);

    // Parse the enum
    let enm = parse_macro_input!(item as ItemEnum);
    let lexer_data = parse_attributes(&enm);

    let enum_name = lexer_data.enum_name;
    let error_token = lexer_data.err_variant;
    let end_token = lexer_data.end_variant;

    // Now we have the regexes, let's construct a DFA
    let mut nfa = nfa::Automaton::new();
    for TokenDefinition{ variant_ident, regex_str, precedence, ignore } in lexer_data.tokens {
        let regex_ast = regex::parse(&regex_str).expect("Error in regex syntax!"); // TODO: Good error msg
        nfa.add_regex_with_accepting_value(&regex_ast, AcceptingState{ variant_ident, precedence, ignore });
    }

    // Determinize the state machine
    let dfa = dfa::Automaton::from_nfa(nfa, |a, b| {
        if a.precedence > b.precedence {
            a
        }
        else if a.precedence < b.precedence {
            b
        }
        else {
            panic!("{} and {} are conflicting!", a.variant_ident, b.variant_ident);
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
                    Some(AcceptingState{ variant_ident, precedence: _, ignore: false }) => quote!{
                        last_accepting = Some((last_lex_state.clone(), Some(#enum_name::#variant_ident)))
                    },
                    Some(AcceptingState{ variant_ident: _, precedence: _, ignore: true }) => quote!{
                        last_accepting = Some((last_lex_state.clone(), None))
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
                if let Some((state, kind)) = last_accepting {
                    // We succeeded before, return that
                    return (state, kind, last_lex_state.source_index);
                }
                else if first_lex_state.is_some() {
                    // No success before, return an error
                    return (first_lex_state.unwrap(), Some(#enum_name::#error_token), last_lex_state.source_index);
                }
                else {
                    // Nothing consumed, no more characters, it's just the end on input
                    return (lex_state.clone(), Some(#enum_name::#end_token), last_lex_state.source_index);
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
    let res = quote!{
        impl ::#FRONT_LIBRARY::TokenType for #enum_name {
            fn is_end(&self) -> bool {
                match self {
                    #enum_name::#end_token => true,
                    _ => false,
                }
            }

            fn next_lexeme_internal(src: &str, lex_state: &::#FRONT_LIBRARY::LexerState) -> (::#FRONT_LIBRARY::LexerState, Option<Self>, usize) {
                let start_idx = lex_state.source_index;
                let source = &src[start_idx..];
                let mut source_it = source.char_indices();
                let mut current_state = #initial_state_id; // State machine state

                let mut last_accepting = None; // Option<(state, Option<token>)>
                let mut first_lex_state = None; // Option<state>
                let mut last_lex_state = lex_state.clone();

                loop {
                    if let Some((current_index, current_char)) = source_it.next() {
                        let current_char_len = current_char.len_utf8();

                        // Update last lex state's position
                        match (last_lex_state.last_char, current_char) {
                            // Newlines
                              (Some('\r'), '\n')
                            | (Some('\r'), _)
                            | (_, '\n') => {
                                last_lex_state.position.newline();
                            },

                            // Any other character
                            (_, ch) => {
                                if !ch.is_control() {
                                    last_lex_state.position.advance_columns(1);
                                }
                            }
                        }
                        // Update last lex state's index
                        last_lex_state.source_index += current_char_len;
                        // Update last lex state's last character
                        last_lex_state.last_char = Some(current_char);

                        // Save if first
                        if first_lex_state.is_none() {
                            first_lex_state = Some(last_lex_state.clone());
                        }

                        match current_state {
                            #(#state_transitions)*
                        }
                    }
                    else {
                        if let Some((state, kind)) = last_accepting {
                            // We succeeded before, return that
                            return (state, kind, last_lex_state.source_index);
                        }
                        else if first_lex_state.is_some() {
                            // No success before, return an error
                            return (first_lex_state.unwrap(), Some(#enum_name::#error_token), last_lex_state.source_index);
                        }
                        else {
                            // Nothing consumed, no more characters, it's just the end on input
                            return (lex_state.clone(), Some(#enum_name::#end_token), last_lex_state.source_index);
                        }
                    }
                }
            }
        }
    };
    //println!("{}", res);
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
        let mut current_def = None;

        for attr in &variant.attrs {
            if attr.path.is_ident(ATTRIBUTE_END) {
                assert!(end_variant.is_none(), "Exactly on 'end' variant must be defined!");
                assert!(current_def.is_none(), "'end' mustn't stand along with any other attribute!");
                end_variant = Some(variant_ident.clone());
            }
            else if attr.path.is_ident(ATTRIBUTE_ERR) {
                assert!(end_variant.is_none(), "Exactly on 'err' variant must be defined!");
                assert!(current_def.is_none(), "'err' mustn't stand along with any other attribute!");
                err_variant = Some(variant_ident.clone());
            }
            else if attr.path.is_ident(ATTRIBUTE_TOKEN) {
                // TODO: Ease this limitation
                // We should allow things like #[token("if"), token("If")]
                assert!(current_def.is_none(), "For now only one definition per attribute!");
                // TODO: Allow '=' too
                let token = attr.parse_args::<LitStr>().unwrap();
                let regex_str = regex::escape(&token.value());
                current_def = Some(TokenDefinition{
                    variant_ident: variant_ident.clone(),
                    regex_str,
                    precedence: 1,
                    ignore: false,
                });
            }
            else if attr.path.is_ident(ATTRIBUTE_C_IDENT) {
                // TODO: Ease this limitation
                // We should allow things like #[token("if"), token("If")]
                assert!(current_def.is_none(), "For now only one definition per attribute!");
                assert!(attr.tokens.is_empty(), "'c_ident' requires no arguments!");
                current_def = Some(TokenDefinition{
                    variant_ident: variant_ident.clone(),
                    regex_str: C_IDENT_REGEX.into(),
                    precedence: 0,
                    ignore: false,
                });
            }
            else if attr.path.is_ident(ATTRIBUTE_REGEX) {
                // TODO: Ease this limitation
                // We should allow things like #[token("if"), token("If")]
                assert!(current_def.is_none(), "For now only one definition per attribute!");
                // TODO: Allow '=' too
                let token = attr.parse_args::<LitStr>().unwrap();
                let regex_str = token.value();
                current_def = Some(TokenDefinition{
                    variant_ident: variant_ident.clone(),
                    regex_str,
                    precedence: 0,
                    ignore: false,
                });
            }
            else if attr.path.is_ident(ATTRIBUTE_IGNORE) {
                let mut cd = current_def.expect("'ignore' must be attached to a token definition!");
                cd.ignore = true;
                current_def = Some(cd);
            }
        }

        if let Some(current_def) = current_def {
            tokens.push(current_def);
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
