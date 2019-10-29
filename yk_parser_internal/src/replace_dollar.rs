/**
 * Replace the $<integer> identifier in the given TokenStream.
 */

use std::collections::HashMap;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use proc_macro2::{ Group, Literal, Punct, Ident };
use quote::TokenStreamExt;

pub fn replace_dollar(src: TokenStream, mappings: &HashMap<usize, String>) -> TokenStream {
    let mut result = TokenStream::new();

    let mut prev_dollar = None;
    for tok in src {
        match tok {
            TokenTree::Group(grp) => {
                dump_dollar(&mut result, &mut prev_dollar);

                // Recurse
                let delim = grp.delimiter();
                let sub = replace_dollar(grp.stream(), mappings);

                result.append(TokenTree::Group(Group::new(delim, sub)));
            },

            TokenTree::Literal(lit) => {
                // If this is a number and there was a dollar before, substitute
                let lit_str = lit.to_string();

                if lit_str.chars().all(|c| c.is_digit(10)) && prev_dollar.is_some() {
                    let lit_int = lit_str.parse::<usize>().unwrap();
                    let subst = mappings.get(&lit_int).unwrap();
                    prev_dollar = None;
                    result.append(TokenTree::Literal(Literal::string(subst)));
                }
                else {
                    // Not number, just dump it
                    dump_dollar(&mut result, &mut prev_dollar);
                    result.append(TokenTree::Literal(lit));
                }
            },

            TokenTree::Punct(punct) => {
                if punct.as_char() == '$' {
                    if dump_dollar(&mut result, &mut prev_dollar) {
                        // The prev. character was a dollar, it escapes this one
                        // NOOP
                    }
                    else {
                        prev_dollar = Some(punct);
                    }
                }
                else {
                    dump_dollar(&mut result, &mut prev_dollar);
                    result.append(TokenTree::Punct(punct));
                }
            },

            // Don't care
            TokenTree::Ident(ident) => {
                result.append(TokenTree::Ident(ident));
            }
        }
    }
    result
}

fn dump_dollar(to: &mut TokenStream, what: &mut Option<Punct>) -> bool {
    if let Some(punct) = what {
        to.append(TokenTree::Punct(punct.clone()));
        *what = None;
        true
    }
    else {
        false
    }
}
