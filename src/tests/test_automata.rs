#[cfg(test)]
mod test_automata {
    use petgraph::visit::IntoEdges;

    use crate::{set, structs::finite_automata::*};
    use std::collections::BTreeSet;

    #[test]
    fn check1() {
        let edges = vec![
            ("s0", ' ', "s1"),
            ("s1", 'a', "s1"),
            ("s1", 'a', "s2"),
            ("s1", 'b', "s1"),
            ("s2", 'b', "s3"),
            ("s3", 'b', "s4"),
        ];

        let nfa = FiniteAutomata::from_slice(&edges, "s0", &vec!["s4"]);
        nfa.export("nfa1");
        let dfa = nfa.to_dfa();
        dfa.export("dfa1");
    }

    #[test]
    fn check2() {
        let edges = vec![
            ("q0", 'a', "q1"), ("q1", ' ', "q2"),
            ("q2", ' ', "q3"), ("q2", ' ', "q9"),
            ("q3", ' ', "q4"), ("q3", ' ', "q6"),
            ("q4", 'b', "q5"), ("q6", 'c', "q7"),
            ("q5", ' ', "q8"), ("q7", ' ', "q8"),
            ("q8", ' ', "q3"), ("q8", ' ', "q9")
        ];
        let nfa = FiniteAutomata::from_slice(&edges, "q0", &vec!["q9"]);
        nfa.export("nfa2");
        let dfa = nfa.to_dfa();
        dfa.export("dfa2");
    }
}
