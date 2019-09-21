/**
 * Generic dense deterministic finite automaton representation.
 */

use std::collections::{BTreeMap, HashMap, HashSet};
use crate::nfa::Automaton as NFA;
use yk_intervals::{Interval, IntervalMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl <T> Automaton<T> where T : Clone + Ord {
    fn add_transition(&mut self, from: State, on: Interval<T>, to: State) {
        let from_map = self.transitions.entry(from).or_insert(IntervalMap::new());
        from_map.insert_and_unify(on, to, |_| panic!());
    }
}

/**
 * Determinization.
 */

impl <T> From<NFA<T>> for Automaton<T> where T : Clone + Ord {
    fn from(nfa: NFA<T>) -> Self {
        let mut dfa = Self::new();
        let mut nfa_set_to_dfa_state = BTreeMap::new();
        let mut stk = Vec::new();

        // We need the start state's mapping
        {
            let start_states = nfa.epsilon_closure(&nfa.start);
            nfa_set_to_dfa_state.insert(start_states.clone(), dfa.start);
            stk.push((start_states, dfa.start));
        }

        while !stk.is_empty() {
            // (set-of-NFA-states, DFA-state)
            let (nfa_states, dfa_state) = stk.pop().unwrap();
            // Now we collect where we can transition to using an interval map
            let mut transitions = IntervalMap::new();

            for nf_state in nfa_states {
                if let Some(trs) = nfa.transitions_from(&nf_state) {
                    for (iv, dest_states) in trs {
                        transitions.insert_and_unify(iv.clone(), dest_states.clone(), |mut unif| {
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
                    let dfa_state = dfa.unique_state();
                    nfa_set_to_dfa_state.insert(to.clone(), dfa_state);
                    dfa.add_transition(dfa_state, on, dfa_state);
                    stk.push((to, dfa_state));
                }
            }
        }

        dfa
    }
}

#[cfg(test)]
mod testing {
use super::*;

#[test]
fn foo_bar_baz() {
    use yk_intervals::{LowerBound, UpperBound};

    let mut nf = NFA::new();

    let a = nf.start;
    let b = nf.unique_state();
    let c = nf.unique_state();
    let d = nf.unique_state();
    let e = nf.unique_state();
    let f = nf.unique_state();
    let g = nf.unique_state();

    nf.add_transition(b, Interval::with_bounds(LowerBound::Included(1), UpperBound::Included(10)), g);
    nf.add_transition(c, Interval::with_bounds(LowerBound::Included(5), UpperBound::Included(15)), f);
    nf.add_transition(d, Interval::with_bounds(LowerBound::Included(3), UpperBound::Included(7)), e);

    nf.add_epsilon_transition(a, b);
    nf.add_epsilon_transition(a, c);
    nf.add_epsilon_transition(a, d);

    let df = Automaton::from(nf);

    println!("{:?}", df);
}
}
