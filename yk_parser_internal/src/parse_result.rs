/**
 * The result type of a parser.
 */

use std::collections::{HashMap, HashSet};

pub enum ParseResult<T, I> {
    Ok(ParseOk<T, I>),
    Err(ParseErr),
}

pub struct ParseOk<T, I> {
    furthest_look: usize, // = consumed
    furthest_it: I,
    furthest_error: Option<ParseErr>,
    value: T,
}

pub struct ParseErr {
    furthest_look: usize,
    found_element: String,
    // rule name -> element
    elements: HashMap<String, ParseErrElement>,
}

pub struct ParseErrElement {
    rule: String,
    expected_elements: HashSet<String>,
}

impl <T, I> ParseResult<T, I> {
    pub fn is_ok(&self) -> bool {
        match self {
            ParseResult::Ok(_) => true,
            _ => false,
        }
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    pub fn unify_alternatives(a: Self, b: Self) -> Self {
        match (a, b) {
            (ParseResult::Ok(mut a), ParseResult::Ok(mut b)) => {
                if a.furthest_look > b.furthest_look {
                    a.furthest_error = Self::unify_errors_in_oks(a.furthest_error, b.furthest_error);
                    ParseResult::Ok(a)
                }
                else if b.furthest_look > a.furthest_look {
                    b.furthest_error = Self::unify_errors_in_oks(a.furthest_error, b.furthest_error);
                    ParseResult::Ok(b)
                }
                else {
                    // TODO: Warn about ambiguity?
                    a.furthest_error = Self::unify_errors_in_oks(a.furthest_error, b.furthest_error);
                    ParseResult::Ok(a)
                }
            },

              (ParseResult::Ok(mut a), ParseResult::Err(b))
            | (ParseResult::Err(b), ParseResult::Ok(mut a)) => {
                a.furthest_error = Some(Self::unify_errors_in_ok_err(a.furthest_error, b));
                ParseResult::Ok(a)
            },

            (ParseResult::Err(a), ParseResult::Err(b)) =>
                ParseResult::Err(Self::unify_errors(a, b)),
        }
    }

    pub fn unify_sequence<U>(a: ParseOk<T, I>, b: ParseResult<U, I>) -> ParseResult<(T, U), I> {
        match b {
            ParseResult::Ok(b) => {
                ParseResult::Ok(ParseOk{
                    furthest_look: b.furthest_look,
                    furthest_it: b.furthest_it,
                    furthest_error: Self::unify_errors_in_oks(a.furthest_error, b.furthest_error),
                    value: (a.value, b.value),
                })
            },

            ParseResult::Err(b) => {
                ParseResult::Err(Self::unify_errors_in_ok_err(a.furthest_error, b))
            }
        }
    }

    fn unify_errors_in_oks(a: Option<ParseErr>, b: Option<ParseErr>) -> Option<ParseErr> {
        if a.is_some() && b.is_some() {
            let aerr = a.unwrap();
            let berr = b.unwrap();
            Some(Self::unify_errors(aerr, berr))
        }
        else if a.is_some() {
            a
        }
        else if b.is_some() {
            b
        }
        else {
            None
        }
    }

    fn unify_errors_in_ok_err(a: Option<ParseErr>, b: ParseErr) -> ParseErr {
        if a.is_some() {
            let aerr = a.unwrap();
            Self::unify_errors(aerr, b)
        }
        else {
            b
        }
    }

    fn unify_errors(a: ParseErr, b: ParseErr) -> ParseErr {
        if a.furthest_look > b.furthest_look {
            a
        }
        else if b.furthest_look > a.furthest_look {
            b
        }
        else {
            // We unify the errors
            assert!(a.found_element == b.found_element);

            let mut elements = a.elements;
            for (k, v) in b.elements {
                if elements.contains_key(&k) {
                    let err = elements.get_mut(&k).unwrap();
                    err.expected_elements.extend(v.expected_elements);
                }
                else {
                    elements.insert(k, v);
                }
            }

            ParseErr{
                furthest_look: a.furthest_look,
                found_element: a.found_element,
                elements
            }
        }
    }
}
