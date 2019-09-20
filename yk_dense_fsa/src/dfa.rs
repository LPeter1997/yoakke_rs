/**
 * Generic dense deterministic finite automaton representation.
 */

use std::collections::{BTreeMap, HashMap, HashSet};
use crate::nfa::Automaton as NFA;
use yk_intervals::{Interval, IntervalMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State(usize);

pub struct Automaton<T> {
    state_counter: usize,
    pub start: State,
    accepting: HashSet<State>,
    transitions: HashMap<State, IntervalMap<T, State>>,
}

impl <T> Automaton<T> {
    pub fn new() -> Self {
        Self{
            state_counter: 0,
            start: State(0),
            accepting: HashSet::new(),
            transitions: HashMap::new(),
        }
    }

    pub fn unique_state(&mut self) -> State {
        self.state_counter += 1;
        State(self.state_counter)
    }

    pub fn is_accepting(&self, state: &State) -> bool {
        self.accepting.contains(state)
    }
}

/**
 * Determinization.
 */

impl <T> From<NFA<T>> for Automaton<T> {
    fn from(nfa: NFA<T>) -> Self {
        let mut dfa = Self::new();
        let mut nfa_set_to_dfa_state = BTreeMap::new();
        let mut stk = Vec::new();

        // We need the start state's mapping
        {
            let start_states = nfa.epsilon_closure(&nfa.start);
            let first = nfa_set_to_dfa_state.entry(start_states).or_insert(dfa.start);
            stk.push(first);
        }

        while !stk.is_empty() {
            let top = stk.pop().unwrap();
            // TODO
        }

        dfa
    }
}
