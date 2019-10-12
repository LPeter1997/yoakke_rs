/**
 * The result type of a parser.
 */

use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub enum ParseResult<I, T> {
    Ok(ParseOk<I, T>),
    Err(ParseErr),
}

#[derive(Clone)]
pub struct ParseOk<I, T> {
    pub furthest_look: usize, // = consumed
    pub furthest_it: I,
    pub furthest_error: Option<ParseErr>,
    pub value: T,
}

#[derive(Clone)]
pub struct ParseErr {
    pub furthest_look: usize,
    pub found_element: String,
    // rule name -> element
    pub elements: HashMap<String, ParseErrElement>,
}

#[derive(Clone)]
pub struct ParseErrElement {
    pub rule: String,
    pub expected_elements: HashSet<String>,
}

impl <I, T> ParseResult<I, T> {
    pub fn is_ok(&self) -> bool {
        match self {
            ParseResult::Ok(_) => true,
            _ => false,
        }
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    pub fn err(self) -> ParseErr {
        match self {
            ParseResult::Err(err) => err,
            _ => panic!("Wrong alternative!"),
        }
    }

    pub fn ok(self) -> ParseOk<I, T> {
        match self {
            ParseResult::Ok(ok) => ok,
            _ => panic!("Wrong alternative!"),
        }
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

    pub fn unify_sequence<U>(a: ParseOk<I, T>, b: ParseResult<I, U>) -> ParseResult<I, (T, U)> {
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
            assert_eq!(a.found_element, b.found_element);

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

impl ParseErr {
    pub fn single(furthest_look: usize, found_element: String, rule: String, expected_element: String) -> Self {
        let mut elements = HashMap::new();
        elements.insert(rule.clone(), ParseErrElement::single(rule, expected_element));
        Self{ furthest_look, found_element, elements }
    }
}

impl ParseErrElement {
    pub fn single(rule: String, element: String) -> Self {
        let mut expected_elements = HashSet::new();
        expected_elements.insert(element);
        Self{ rule, expected_elements }
    }
}
