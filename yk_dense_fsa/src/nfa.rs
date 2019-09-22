/**
 * Generic dense nondeterministic finite automaton representation.
 */

use std::collections::{BTreeMap, BTreeSet};
use yk_intervals::{Interval, IntervalMap, IntervalSet, LowerBound, UpperBound};
use yk_regex_parse as regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct State(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Automaton<T, AcceptingValue = ()> {
    state_counter: usize,
    pub start: State,
    accepting: BTreeMap<State, AcceptingValue>,
    transitions: BTreeMap<State, IntervalMap<T, BTreeSet<State>>>,
    epsilon: BTreeMap<State, BTreeSet<State>>,
}

impl <T, AcceptingValue> Automaton<T, AcceptingValue> {
    pub fn new() -> Self {
        Self{
            state_counter: 0,
            start: State(0),
            accepting: BTreeMap::new(),
            transitions: BTreeMap::new(),
            epsilon: BTreeMap::new(),
        }
    }

    pub fn unique_state(&mut self) -> State {
        self.state_counter += 1;
        State(self.state_counter)
    }

    pub fn add_accepting_with_value(&mut self, state: State, value: AcceptingValue) {
        self.accepting.insert(state, value);
    }

    pub fn accepting_value(&self, state: &State) -> Option<&AcceptingValue> {
        self.accepting.get(state)
    }

    pub fn is_accepting(&self, state: &State) -> bool {
        self.accepting_value(state).is_some()
    }

    // TODO: Return references (HashSet<&State>) instead? Makes more sense.
    pub fn epsilon_closure(&self, state: &State) -> BTreeSet<State> {
        let mut result = BTreeSet::new();
        let mut touched = BTreeSet::new();

        let mut stk = vec![state];
        while !stk.is_empty() {
            let top = stk.pop().unwrap();
            result.insert(*top);

            if let Some(states) = self.epsilon.get(top) {
                for s in states {
                    if !touched.contains(s) {
                        touched.insert(s);
                        stk.push(s);
                    }
                }
            }
        }

        result
    }

    pub fn add_epsilon_transition(&mut self, from: State, to: State) {
        let from_map = self.epsilon.entry(from).or_insert(BTreeSet::new());
        from_map.insert(to);
    }

    pub fn transitions_from(&self, from: &State) -> Option<&IntervalMap<T, BTreeSet<State>>> {
        self.transitions.get(from)
    }

    pub fn states(&self) -> std::iter::Map<std::ops::RangeInclusive<usize>, fn(usize) -> State> {
        (0..=self.state_counter).map(|x| State(x))
    }
}

impl <T, AcceptingValue> Automaton<T, AcceptingValue> where AcceptingValue : Default {
    pub fn add_accepting(&mut self, state: State) {
        self.add_accepting_with_value(state, AcceptingValue::default());
    }
}

impl <T, AcceptingValue> Automaton<T, AcceptingValue> where T : Clone + Ord {
    pub fn add_transition(&mut self, from: State, on: Interval<T>, to: State) {
        let from_map = self.transitions.entry(from).or_insert(IntervalMap::new());
        let mut hs = BTreeSet::new();
        hs.insert(to);
        from_map.insert_and_unify(on, hs, |mut unification| {
            for v in unification.inserted {
                unification.existing.insert(v);
            }
            unification.existing
        });
    }
}

/**
 * Thompson's-construction.
 */

impl <AcceptingValue> Automaton<char, AcceptingValue> {
    pub fn add_regex_with_accepting_value(
        &mut self, rx: &regex::Node, value: AcceptingValue) -> (State, State) {

        let (from, to) = thompson_construct(self, rx);
        self.add_epsilon_transition(self.start, from);
        self.add_accepting_with_value(to, value);
        (from, to)
    }
}

impl <AcceptingValue> Automaton<char, AcceptingValue> where AcceptingValue : Default {
    pub fn add_regex(&mut self, rx: &regex::Node) -> (State, State) {
        let (from, to) = thompson_construct(self, rx);
        self.add_epsilon_transition(self.start, from);
        self.add_accepting_with_value(to, Default::default());
        (from, to)
    }
}

impl <AcceptingValue> From<regex::Node> for Automaton<char, AcceptingValue> where AcceptingValue : Default {
    fn from(rx: regex::Node) -> Self {
        let mut nf = Self::new();
        nf.add_regex(&rx);
        nf
    }
}

fn thompson_construct<AcceptingValue>(nfa: &mut Automaton<char, AcceptingValue>,
    rx: &regex::Node) -> (State, State) {

    match rx {
        regex::Node::Alternative{ first, second } =>
            thompson_construct_alternative(nfa, first, second),

        regex::Node::Sequence{ first, second } =>
            thompson_construct_sequence(nfa, first, second),

        regex::Node::Quantified{ subnode, quantifier } =>
            thompson_construct_quantified(nfa, subnode, *quantifier),

        regex::Node::Grouping{ negated, elements } =>
            thompson_construct_grouping(nfa, *negated, elements),

        regex::Node::Literal(ch) =>
            thompson_construct_literal(nfa, *ch),
    }
}

fn thompson_construct_alternative<AcceptingValue>(nfa: &mut Automaton<char, AcceptingValue>,
    left: &regex::Node, right: &regex::Node) -> (State, State) {

    let start = nfa.unique_state();
    let end = nfa.unique_state();

    let (l_s, l_e) = thompson_construct(nfa, left);
    let (r_s, r_e) = thompson_construct(nfa, right);

    nfa.add_epsilon_transition(start, l_s);
    nfa.add_epsilon_transition(start, r_s);

    nfa.add_epsilon_transition(l_e, end);
    nfa.add_epsilon_transition(r_e, end);

    (start, end)
}

fn thompson_construct_sequence<AcceptingValue>(nfa: &mut Automaton<char, AcceptingValue>,
    left: &regex::Node, right: &regex::Node) -> (State, State) {

    let (l_s, l_e) = thompson_construct(nfa, left);
    let (r_s, r_e) = thompson_construct(nfa, right);

    nfa.add_epsilon_transition(l_e, r_s);

    (l_s, r_e)
}

fn thompson_construct_quantified<AcceptingValue>(nfa: &mut Automaton<char, AcceptingValue>,
    subnode: &regex::Node, quantifier: regex::Quantifier) -> (State, State) {

    match quantifier {
        regex::Quantifier::AtLeast(count) => {
            let (start, end) = thompson_construct_repeat(nfa, subnode, count);

            // Allow looping on the last state
            let (ls, le) = thompson_construct(nfa, subnode);
            nfa.add_epsilon_transition(end, ls);
            nfa.add_epsilon_transition(le, end);

            (start, end)
        },

        regex::Quantifier::Between(least, most) => {
            assert!(least <= most);

            let (start, min) = thompson_construct_repeat(nfa, subnode, least);
            let mut last = min;
            let end = nfa.unique_state();

            // From here we let every node skip to the end with an epsilon-transition
            nfa.add_epsilon_transition(last, end);
            for _ in 0..(most - least) {
                let (s, e) = thompson_construct(nfa, subnode);
                nfa.add_epsilon_transition(last, s);
                last = e;
                nfa.add_epsilon_transition(last, end);
            }

            (start, end)
        }
    }
}

fn thompson_construct_grouping<AcceptingValue>(nfa: &mut Automaton<char, AcceptingValue>,
    negated: bool, elements: &Vec<regex::GroupingElement>) -> (State, State) {

    let mut ivs = IntervalSet::new();

    for elem in elements {
        match elem {
            regex::GroupingElement::Literal(ch) => {
                ivs.insert(Interval::singleton(*ch));
            },

            regex::GroupingElement::Range(cfrom, cto) => {
                ivs.insert(Interval::with_bounds(LowerBound::Included(*cfrom), UpperBound::Included(*cto)));
            },
        }
    }

    if negated {
        ivs.invert();
    }

    let start = nfa.unique_state();
    let end = nfa.unique_state();

    for iv in ivs {
        nfa.add_transition(start, iv, end);
    }

    (start, end)
}

fn thompson_construct_literal<AcceptingValue>(nfa: &mut Automaton<char, AcceptingValue>,
    ch: char) -> (State, State) {

    let start = nfa.unique_state();
    let end = nfa.unique_state();

    nfa.add_transition(start, Interval::singleton(ch), end);

    (start, end)
}

fn thompson_construct_repeat<AcceptingValue>(nfa: &mut Automaton<char, AcceptingValue>,
    node: &regex::Node, count: usize) -> (State, State) {

    let start = nfa.unique_state();
    let mut last = start;

    for _ in 0..count {
        let (s, e) = thompson_construct(nfa, node);
        nfa.add_epsilon_transition(last, s);
        last = e;
    }

    (start, last)
}

// TODO: An NFA could be constructed trivially from a DFA (so we could have From<DFA>)
// implemented here. That could be used for some optimizations.
