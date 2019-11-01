/**
 * The result type of a parser.
 */

use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub enum ParseResult<T, E> {
    Ok(ParseOk<T, E>),
    Err(ParseErr<E>),
}

#[derive(Clone)]
pub struct ParseOk<T, E> {
    pub matched: usize,
    pub furthest_error: Option<ParseErr<E>>,
    pub value: T,
}

#[derive(Clone)]
pub struct ParseErr<E> {
    pub furthest_look: usize,
    pub found_element: Found<E>,
    // rule name -> element
    pub elements: HashMap<&'static str, ParseErrElement>,
}

#[derive(Clone)]
pub struct ParseErrElement {
    pub rule: &'static str,
    pub expected_elements: HashSet<String>,
}

#[derive(Clone)]
pub enum Found<E> {
    Element(E),
    EndOfInput,
    Stub,
}

#[derive(Clone, Copy)]
pub struct EndOfInput;

impl <T, E> ParseResult<T, E> {
    pub fn is_ok(&self) -> bool {
        match self {
            ParseResult::Ok(_) => true,
            _ => false,
        }
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    pub fn err(self) -> Option<ParseErr<E>> {
        match self {
            ParseResult::Err(err) => Some(err),
            _ => None,
        }
    }

    pub fn ok(self) -> Option<ParseOk<T, E>> {
        match self {
            ParseResult::Ok(ok) => Some(ok),
            _ => None,
        }
    }

    pub fn map<F, U>(self, f: F) -> ParseResult<U, E> where F: FnOnce(T) -> U {
        if let ParseResult::Ok(ok) = self {
            ok.map(f).into()
        }
        else {
            self.err().unwrap().into()
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
                let err = Self::unify_alternative_errors_opt(a.furthest_error, b.furthest_error);
                if a.matched > b.matched {
                    a.furthest_error = err;
                    ParseResult::Ok(a)
                }
                else if b.matched > a.matched {
                    b.furthest_error = err;
                    ParseResult::Ok(b)
                }
                else {
                    // TODO: Warn about ambiguity?
                    // We arbitrarily choose the first one here
                    a.furthest_error = err;
                    ParseResult::Ok(a)
                }
            },

              (ParseResult::Ok(mut a), ParseResult::Err(b))
            | (ParseResult::Err(b), ParseResult::Ok(mut a)) => {
                let err = Self::unify_alternative_errors_opt(a.furthest_error, Some(b));
                assert!(err.is_some());
                a.furthest_error = err;
                ParseResult::Ok(a)
            },

            (ParseResult::Err(a), ParseResult::Err(b)) =>
                ParseResult::Err(Self::unify_alternative_errors(a, b)),
        }
    }

    pub fn unify_sequence<U>(a: ParseOk<T, E>, b: ParseResult<U, E>) -> ParseResult<(T, U), E> {
        match b {
            ParseResult::Ok(b) => {
                ParseResult::Ok(ParseOk{
                    matched: a.matched + b.matched,
                    furthest_error: Self::unify_sequence_errors_opt(a.matched, a.furthest_error, b.furthest_error),
                    value: (a.value, b.value),
                })
            },

            ParseResult::Err(b) => {
                let err = Self::unify_sequence_errors_opt(a.matched, a.furthest_error, Some(b));
                assert!(err.is_some());
                ParseResult::Err(err.unwrap())
            }
        }
    }

    fn unify_sequence_errors_opt(am: usize, a: Option<ParseErr<E>>, b: Option<ParseErr<E>>) -> Option<ParseErr<E>> {
        // Add the match offset to the right-hand side
        let b = match b {
            Some(mut b) => {
                b.furthest_look += am;
                Some(b)
            }
            None => None,
        };

        match (a, b) {
            (Some(a), Some(b)) => Some(Self::unify_sequence_errors(a, b)),

              (Some(a), None)
            | (None, Some(a)) => Some(a),

            (None, None) => None,
        }
    }

    fn unify_alternative_errors_opt(a: Option<ParseErr<E>>, b: Option<ParseErr<E>>) -> Option<ParseErr<E>> {
        match (a, b) {
            (Some(a), Some(b)) => Some(Self::unify_alternative_errors(a, b)),

              (Some(a), None)
            | (None, Some(a)) => Some(a),

            (None, None) => None,
        }
    }

    // TODO: Same things

    fn unify_alternative_errors(a: ParseErr<E>, b: ParseErr<E>) -> ParseErr<E> {
        if a.furthest_look > b.furthest_look {
            a
        }
        else if b.furthest_look > a.furthest_look {
            b
        }
        else {
            // We unify the errors
            Self::unify_errors(a, b)
        }
    }

    fn unify_sequence_errors(a: ParseErr<E>, b: ParseErr<E>) -> ParseErr<E> {
        if a.furthest_look > b.furthest_look {
            a
        }
        else if b.furthest_look > a.furthest_look {
            b
        }
        else {
            // We unify the errors
            Self::unify_errors(a, b)
        }
    }

    fn unify_errors(a: ParseErr<E>, b: ParseErr<E>) -> ParseErr<E> {
        // Special case if one is an empty element
        if let Found::Stub = a.found_element {
            b
        }
        else if let Found::Stub = b.found_element {
            a
        }
        else {
            let mut elements = a.elements;
            for (k, v) in b.elements {
                // TODO: We could use the entry API?
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

impl <T, E> From<ParseOk<T, E>> for ParseResult<T, E> {
    fn from(ok: ParseOk<T, E>) -> Self {
        Self::Ok(ok)
    }
}

impl <T, E> From<ParseErr<E>> for ParseResult<T, E> {
    fn from(err: ParseErr<E>) -> Self {
        Self::Err(err)
    }
}

impl <T, E> ParseOk<T, E> {
    pub fn furthest_look(&self) -> usize {
        if let Some(err) = &self.furthest_error {
            std::cmp::max(self.matched, err.furthest_look())
        }
        else {
            self.matched
        }
    }

    pub fn map<F, U>(self, f: F) -> ParseOk<U, E> where F: FnOnce(T) -> U {
        ParseOk{
            matched: self.matched,
            furthest_error: self.furthest_error,
            value: f(self.value),
        }
    }
}

impl <E> ParseErr<E> {
    pub fn new() -> Self {
        Self{ furthest_look: 0, found_element: Found::Stub, elements: HashMap::new() }
    }

    pub fn single(furthest_look: usize, found_element: Found<E>, rule: &'static str, expected_element: String) -> Self {
        let mut elements = HashMap::new();
        elements.insert(rule.clone(), ParseErrElement::single(rule, expected_element));
        Self{ furthest_look, found_element, elements }
    }

    pub fn furthest_look(&self) -> usize {
        self.furthest_look
    }

    /*pub fn merge_element_into(&mut self, from_r: &'static str, to_r: &'static str) {
        let from = self.elements.remove(from_r);
        let to = self.elements.get_mut(to_r);

        match (to, from) {
            (Some(to), Some(from)) => to.merge(from),
            (None, Some(from)) => { self.elements.insert(to_r, from); },
            (None, None) | (Some(_), None) => {},
        }
    }*/
}

impl ParseErrElement {
    pub fn single(rule: &'static str, element: String) -> Self {
        let mut expected_elements = HashSet::new();
        expected_elements.insert(element);
        Self{ rule, expected_elements }
    }

    /*fn merge(&mut self, src: ParseErrElement) {
        self.expected_elements.extend(src.expected_elements);
    }*/
}
