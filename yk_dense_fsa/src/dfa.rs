/**
 * Generic dense deterministic finite automaton representation.
 */

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use crate::nfa::Automaton as NFA;
use yk_intervals::{Interval, IntervalMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Automaton<T, AcceptingValue = ()> {
    state_counter: usize,
    pub start: State,
    accepting: HashMap<State, AcceptingValue>,
    transitions: HashMap<State, IntervalMap<T, State>>,
}

impl <T, AcceptingValue> Automaton<T, AcceptingValue> {
    pub fn new() -> Self {
        Self{
            state_counter: 0,
            start: State(0),
            accepting: HashMap::new(),
            transitions: HashMap::new(),
        }
    }

    pub fn unique_state(&mut self) -> State {
        self.state_counter += 1;
        State(self.state_counter)
    }

    pub fn accepting_value(&self, state: &State) -> Option<&AcceptingValue> {
        self.accepting.get(state)
    }

    pub fn is_accepting(&self, state: &State) -> bool {
        self.accepting_value(state).is_some()
    }

    pub fn add_accepting_with_value(&mut self, state: State, value: AcceptingValue) {
        self.accepting.insert(state, value);
    }
}

impl <T, AcceptingValue> Automaton<T, AcceptingValue> where AcceptingValue : Default {
    pub fn add_accepting(&mut self, state: State) {
        self.add_accepting_with_value(state, AcceptingValue::default());
    }
}

impl <T, AcceptingValue> Automaton<T, AcceptingValue> where T : Clone + Ord {
    fn add_transition(&mut self, from: State, on: Interval<T>, to: State) {
        let from_map = self.transitions.entry(from).or_insert(IntervalMap::new());
        from_map.insert_and_unify(on, to, |_| panic!());
    }
}

/**
 * Determinization.
 */

impl <T, AcceptingValue> Automaton<T, AcceptingValue> where T : Clone + Ord, AcceptingValue : Clone {
    pub fn from_nfa<F>(nfa: NFA<T, AcceptingValue>, unify: F) -> Self
        where F : FnMut(AcceptingValue, AcceptingValue) -> AcceptingValue {

        let mut dfa = Self::new();
        let mut nfa_set_to_dfa_state = BTreeMap::new();
        let mut stk = Vec::new();

        // We need the start state's mapping
        {
            let start_states = nfa.epsilon_closure(&nfa.start);
            nfa_set_to_dfa_state.insert(start_states.clone(), dfa.start);
            stk.push((start_states, dfa.start));

            // Accepting registration
            if start_states.iter().any(|x| nfa.is_accepting(&x)) {
                // TODO: Not this
                dfa.add_accepting_with_value(dfa.start, val.clone());
            }
        }

        while !stk.is_empty() {
            // (set-of-NFA-states, DFA-state)
            let (nfa_states, dfa_state) = stk.pop().unwrap();
            // Now we collect where we can transition to using an interval map
            let mut transitions = IntervalMap::new();

            for nf_state in nfa_states {
                if let Some(trs) = nfa.transitions_from(&nf_state) {
                    for (iv, dest_states) in trs {
                        // Expand 'dest_states' with epsilon closure
                        let mut ds = BTreeSet::new();
                        for s in dest_states {
                            ds.extend(nfa.epsilon_closure(&s));
                        }

                        transitions.insert_and_unify(iv.clone(), ds, |mut unif| {
                            unif.existing.extend(unif.inserted);
                            unif.existing
                        });
                    }
                }
            }

            // Now 'transitions' contains all transitions from the set of nfa states
            for (on, to) in transitions {
                if let Some(dfa_to) = nfa_set_to_dfa_state.get(&to) {
                    dfa.add_transition(dfa_state, on, *dfa_to);
                }
                else {
                    let dfa_state_to = dfa.unique_state();

                    // Accepting registration
                    if to.iter().any(|x| nfa.is_accepting(&x)) {
                        // TODO: Not this
                        dfa.add_accepting(dfa_state_to);
                    }

                    nfa_set_to_dfa_state.insert(to.clone(), dfa_state_to);
                    dfa.add_transition(dfa_state, on, dfa_state_to);
                    stk.push((to, dfa_state_to));
                }
            }
        }

        dfa
    }
}

// TODO: Tests
