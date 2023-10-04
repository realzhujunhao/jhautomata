use std::{
    collections::{BTreeSet, HashSet},
    fmt::Display,
};

use rayon::prelude::*;

/// HashSet does not impl Hash, therefore HashSet<HashSet<String>> is illegal
/// use HashSet<BTreeSet> instead!
type State = BTreeSet<String>;
type Transition = (State, char, State);
pub const EPSILON: char = ' ';

pub struct FiniteAutomata {
    pub states: HashSet<State>,
    pub transitions: HashSet<Transition>,
    pub final_states: HashSet<State>,
    pub start_state: State,
}

impl FiniteAutomata {
    /// accept &str slices and omit states for convenience
    /// construct states based on transitions in correct type
    pub fn new(
        transitions: HashSet<(&str, char, &str)>,
        final_states: HashSet<&str>,
        start_state: &str,
    ) -> Self {
        // states: all distinct states that appear in transitions
        let states = transitions
            .par_iter()
            .flat_map(|(from, _, to)| {
                vec![
                    BTreeSet::from_iter(vec![(*from).into()]),
                    BTreeSet::from_iter(vec![(*to).into()]),
                ]
            })
            .collect();
        // transitions: (&str, char, &str) -> (BTreeSet<String>, char, BTreeSet<String>)
        let transitions = transitions
            .into_par_iter()
            .map(|(from, symbol, to)| {
                (
                    BTreeSet::from_iter(vec![from.into()]),
                    symbol,
                    BTreeSet::from_iter(vec![to.into()]),
                )
            })
            .collect();
        // final_states: HashSet<&str> -> HashSet<BTreeSet<String>>
        let final_states = final_states
            .par_iter()
            .map(|&s| BTreeSet::from_iter(vec![s.into()]))
            .collect();
        // start_state: &str -> BTreeSet<String>
        let start_state = BTreeSet::from_iter(vec![start_state.into()]);
        Self {
            states,
            transitions,
            final_states,
            start_state,
        }
    }

    /// false if there exists epsilon or duplicated transition
    /// true otherwise
    pub fn is_deterministic(&self) -> bool {
        let mut dup_check = HashSet::new();
        for (from, symbol, _) in &self.transitions {
            if !dup_check.insert((from, *symbol)) || *symbol == EPSILON {
                return false;
            }
        }
        true
    }

    /// construct a DFA based on &self
    pub fn subset_construction(&self) -> Self {
        let mut states = HashSet::new();
        let mut transitions = HashSet::new();
        let mut final_states = HashSet::new();
        let start_state = self.epsilon_closure(&self.start_state);

        let mut process_set = HashSet::new();
        process_set.insert(start_state.clone());

        while !process_set.is_empty() {
            let from = process_set.iter().next().unwrap().clone();
            if states.contains(&from) {
                process_set.remove(&from);
                continue;
            }
            let mut symbols = self.next_alphabets(&from);
            symbols.remove(&EPSILON);
            for symbol in symbols {
                let target_state = self.reachable_states(&from, symbol);
                let target_state = self.epsilon_closure(&target_state);
                process_set.insert(target_state.clone());
                transitions.insert((from.clone(), symbol, target_state));
            }
            states.insert(process_set.take(&from).unwrap());
        }

        // if state and original final state have intersection
        // then it becomes new final state
        for s in &states {
            if self.final_states.par_iter().any(|f| f.is_subset(s)) {
                final_states.insert(s.clone());
            }
        }
        Self {
            states,
            transitions,
            final_states,
            start_state,
        }
    }

    /// return all states that are reachable from epsilon transition + original states
    fn epsilon_closure(&self, from: &State) -> State {
        let mut result = from.clone();
        let mut epsilon_move = from.clone();
        loop {
            let prev = epsilon_move.clone();
            epsilon_move = self.reachable_states(&epsilon_move, EPSILON);
            epsilon_move.retain(|x| !prev.contains(x));
            if epsilon_move.is_empty() {
                break;
            }
            result.extend(epsilon_move.clone());
        }
        result
    }

    /// find all states that are reachable from the given state by consuming the given symbol
    fn reachable_states(&self, from: &State, symbol: char) -> State {
        let mut reachable = State::new();
        for (a, b, c) in &self.transitions {
            if a.is_subset(from) && symbol == *b {
                reachable.extend(c.clone());
            }
        }
        reachable
    }

    /// find all characters that moves the given state to another valid state
    fn next_alphabets(&self, state: &State) -> HashSet<char> {
        self.transitions
            .par_iter()
            .filter_map(|(from, symbol, _)| {
                if from.is_subset(state) {
                    Some(*symbol)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Display for FiniteAutomata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut transition = String::new();
        for (a, b, c) in &self.transitions {
            transition.push_str(&format!("{:?} - {} -> {:?}\n", a, b, c));
        }
        write!(
            f,
            "\n States: {:?} \nTransitions\n{} \nFinal States{:?} \nStart State: {:?}",
            self.states, transition, self.final_states, self.start_state
        )
    }
}
