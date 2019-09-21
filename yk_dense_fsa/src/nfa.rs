/**
 * Generic dense nondeterministic finite automaton representation.
 */

use std::collections::{BTreeMap, BTreeSet};
use yk_intervals::{Interval, IntervalMap};
use yk_regex_parse as regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct State(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Automaton<T> {
    state_counter: usize,
    pub start: State,
    accepting: BTreeSet<State>,
    transitions: BTreeMap<State, IntervalMap<T, BTreeSet<State>>>,
    epsilon: BTreeMap<State, BTreeSet<State>>,
}

impl <T> Automaton<T> {
    pub fn new() -> Self {
        Self{
            state_counter: 0,
            start: State(0),
            accepting: BTreeSet::new(),
            transitions: BTreeMap::new(),
            epsilon: BTreeMap::new(),
        }
    }

    pub fn unique_state(&mut self) -> State {
        self.state_counter += 1;
        State(self.state_counter)
    }

    pub fn add_accepting(&mut self, state: State) {
        self.accepting.insert(state);
    }

    pub fn is_accepting(&self, state: &State) -> bool {
        self.accepting.contains(state)
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
}

impl <T> Automaton<T> where T : Clone + Ord {
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

impl From<regex::Node> for Automaton<char> {
    fn from(rx: regex::Node) -> Self {
        let mut nf = Automaton::new();
        let (from, to) = thompson_construct(&mut nf, rx);
        nf.add_epsilon_transition(nf.start, from);
        nf.add_accepting(to);
        nf
    }
}

fn thompson_construct(nfa: &mut Automaton<char>,
    rx: regex::Node) -> (State, State) {

    unimplemented!();
}

fn thompson_construct_alternative(nfa: &mut Automaton<char>,
    left: regex::Node, right: regex::Node) -> (State, State) {

    unimplemented!();
}

fn thompson_construct_sequence(nfa: &mut Automaton<char>,
    left: regex::Node, right: regex::Node) -> (State, State) {

    unimplemented!();
}

fn thompson_construct_quantified(nfa: &mut Automaton<char>,
    subnode: regex::Node, quantifier: regex::Quantifier) -> (State, State) {

    unimplemented!();
}

fn thompson_construct_grouping(nfa: &mut Automaton<char>,
    negated: bool, elements: Vec<regex::GroupingElement>) -> (State, State) {

    unimplemented!();
}

fn thompson_construct_literal(nfa: &mut Automaton<char>,
    ch: char) -> (State, State) {

    unimplemented!();
}
