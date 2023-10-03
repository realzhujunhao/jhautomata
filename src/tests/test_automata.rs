#[cfg(test)]
mod test_automata {
    use std::collections::HashSet;

    use crate::structs::finite_automata::{FiniteAutomata, EPSILON};

    // #[test]
    // fn subset_construction_1() {
    //     let transitions = HashSet::from_iter(vec![
    //         ("s0", EPSILON, "s1"),
    //         ("s1", 'a', "s1"),
    //         ("s1", 'a', "s2"),
    //         ("s1", 'b', "s1"),
    //         ("s2", 'b', "s3"),
    //         ("s3", 'b', "s4"),
    //     ]);
    //     let final_states = HashSet::from_iter(vec!["s4"]);
    //     let nfa = FiniteAutomata::new(transitions, final_states, "s0");
    //     assert_eq!(nfa.is_deterministic(), false);
    //
    //     println!("--- TEST 1 ---");
    //     println!("{}", nfa);
    //     let dfa = nfa.subset_construction();
    //     println!("{}", dfa);
    // }

    #[test]
    fn subset_construction_2() {
        let transitions = HashSet::from_iter(vec![
            ("X", 'a', "X"),
            ("X", 'b', "X"),
            ("X", 'a', "Y"),
            ("X", 'a', "Z"),
            ("Y", 'b', "Y"),
            ("Z", 'c', "Z"),
        ]);
        let final_states = HashSet::from_iter(vec!["Y", "Z"]);
        let nfa = FiniteAutomata::new(transitions, final_states, "X");
        assert_eq!(nfa.is_deterministic(), false);


        println!("--- TEST 2 ---");
        println!("{}", nfa);
        let dfa = nfa.subset_construction();
        println!("{}", dfa);
    }
}
