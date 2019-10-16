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
    pub matched: usize,
    pub furthest_it: I,
    pub furthest_error: Option<ParseErr>,
    pub value: T,
}

#[derive(Clone)]
pub struct ParseErr {
    pub furthest_look: usize,
    pub found_element: String,
    // rule name -> element
    pub elements: HashMap<&'static str, ParseErrElement>,
}

#[derive(Clone)]
pub struct ParseErrElement {
    pub rule: &'static str,
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

    pub fn err(self) -> Option<ParseErr> {
        match self {
            ParseResult::Err(err) => Some(err),
            _ => None,
        }
    }

    pub fn ok(self) -> Option<ParseOk<I, T>> {
        match self {
            ParseResult::Ok(ok) => Some(ok),
            _ => None,
        }
    }

    pub fn furthest_look(&self) -> usize {
        match self {
            ParseResult::Ok(ok) => ok.furthest_look(),
            ParseResult::Err(err) => err.furthest_look(),
        }
    }

    pub fn unify_alternatives(a: Self, b: Self) -> Self {
        match (a, b) {
            (ParseResult::Ok(mut a), ParseResult::Ok(mut b)) => {
                if a.matched > b.matched {
                    a.furthest_error = Self::unify_errors_in_oks(a.furthest_error, b.furthest_error);
                    ParseResult::Ok(a)
                }
                else if b.matched > a.matched {
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
                    matched: b.matched,
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
            // Special case if one is an empty element
            if a.found_element == "" {
                b
            }
            else if b.found_element == "" {
                a
            }
            else {
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
}

impl <I, T> From<ParseOk<I, T>> for ParseResult<I, T> {
    fn from(ok: ParseOk<I, T>) -> Self {
        Self::Ok(ok)
    }
}

impl <I, T> From<ParseErr> for ParseResult<I, T> {
    fn from(err: ParseErr) -> Self {
        Self::Err(err)
    }
}

impl <I, T> ParseOk<I, T> {
    pub fn furthest_look(&self) -> usize {
        if let Some(err) = &self.furthest_error {
            std::cmp::max(self.matched, err.furthest_look())
        }
        else {
            self.matched
        }
    }

    pub fn map<F, U>(self, f: F) -> ParseOk<I, U> where F: FnOnce(T) -> U {
        ParseOk{
            matched: self.matched,
            furthest_it: self.furthest_it,
            furthest_error: self.furthest_error,
            value: f(self.value),
        }
    }
}

impl ParseErr {
    pub fn new() -> Self {
        Self{ furthest_look: 0, found_element: String::new(), elements: HashMap::new() }
    }

    pub fn single(furthest_look: usize, found_element: String, rule: &'static str, expected_element: String) -> Self {
        let mut elements = HashMap::new();
        elements.insert(rule.clone(), ParseErrElement::single(rule, expected_element));
        Self{ furthest_look, found_element, elements }
    }

    pub fn furthest_look(&self) -> usize {
        self.furthest_look
    }
}

impl ParseErrElement {
    pub fn single(rule: &'static str, element: String) -> Self {
        let mut expected_elements = HashSet::new();
        expected_elements.insert(element);
        Self{ rule, expected_elements }
    }
}
