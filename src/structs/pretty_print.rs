use std::{collections::BTreeSet, fmt::{Display, Debug, write}};

use crate::structs::finite_automata::*;

#[derive(Eq, PartialEq, PartialOrd, Ord, Hash, Clone)]
pub struct PrettyState(pub BTreeSet<String>);

impl Display for PrettyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(", "))
    }
}

impl Debug for PrettyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(", "))
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct PrettyTransition(pub char);

impl Debug for PrettyTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
