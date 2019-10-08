/**
 * The result type of a parser.
 */

use std::collections::{HashMap, HashSet};

pub enum ParseResult<T> {
    Ok(ParseOk<T>),
    Err(ParseErr),
}

pub struct ParseOk<T> {
    furthest_look: usize, // = consumed
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

impl <T> ParseResult<T> {
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
            (ParseResult::Ok(a), ParseResult::Ok(b)) => {
                if a.furthest_look > b.furthest_look {
                    ParseResult::Ok(a)
                }
                else if b.furthest_look > a.furthest_look {
                    ParseResult::Ok(b)
                }
                else {
                    // TODO: Warn about ambiguity?
                    ParseResult::Ok(a)
                }
            },

              (ParseResult::Ok(mut a), ParseResult::Err(b))
            | (ParseResult::Err(b), ParseResult::Ok(mut a)) => {
                if a.furthest_look > b.furthest_look {
                    // Drop the error completely, contains no information
                    ParseResult::Ok(a)
                }
                else {
                    // We need to store the error
                    if let Some(err) = a.furthest_error {
                        // We need to unify the errors
                        a.furthest_error = Some(Self::unify_errors(err, b));
                    }
                    else {
                        a.furthest_error = Some(b);
                    }
                    ParseResult::Ok(a)
                }
            },

            (ParseResult::Err(a), ParseResult::Err(b)) =>
                ParseResult::Err(Self::unify_errors(a, b)),
        }
    }

    pub fn unify_sequence(a: ParseOk<T>, b: Self) -> ParseResult<(T, T)> {
        match b {
            ParseResult::Ok(b) => {
                let furthest_error = if a.furthest_error.is_some() && b.furthest_error.is_some() {
                    Some(Self::unify_errors(a.furthest_error.unwrap(), b.furthest_error.unwrap()))
                }
                else if a.furthest_error.is_some() {
                    a.furthest_error
                }
                else if b.furthest_error.is_some() {
                    b.furthest_error
                }
                else {
                    None
                };
                ParseResult::Ok(ParseOk{
                    furthest_look: a.furthest_look,
                    furthest_error,
                    value: (a.value, b.value),
                })
            },

            ParseResult::Err(b) => {
                unimplemented!();
            }
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
