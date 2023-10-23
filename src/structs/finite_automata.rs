use std::{
    collections::{BTreeSet, HashSet},
    fmt::Debug,
    fs::OpenOptions,
    io::{self, Write},
    process::Command,
};

use petgraph::{
    adj::EdgeIndex,
    data::Build,
    dot::{Config, Dot},
    graph::NodeIndex,
    matrix_graph::node_index,
    visit::{IntoEdgeReferences, IntoEdgesDirected, NodeRef},
    Direction::{Incoming, Outgoing},
    Graph,
};

use rayon::prelude::*;

use crate::set;

use super::pretty_print::{PrettyState, PrettyTransition};

type State = BTreeSet<String>;
type NodeGroup = BTreeSet<NodeIndex>;
type Transition = char;
pub const EPSILON: char = '\u{03B5}';

pub struct FiniteAutomata {
    pub graph: Graph<PrettyState, PrettyTransition>,
    pub start: NodeIndex,
    pub fin: NodeGroup,
}

impl FiniteAutomata {
    pub fn new(
        transitions: &[(State, Transition, State)],
        start: State,
        fin: BTreeSet<State>,
    ) -> Self {
        let mut graph = Graph::new();
        transitions.into_iter().for_each(|(src, t, dest)| {
            let t = if *t == ' ' { EPSILON } else { *t };
            Self::add_edge_no_dup(
                &mut graph,
                PrettyState(src.clone()),
                PrettyTransition(t),
                PrettyState(dest.clone()),
            );
        });
        let start = Self::node_idx(&graph, &PrettyState(start)).expect("start node does not exist");
        let fin = fin
            .iter()
            .map(|s| {
                Self::node_idx(&graph, &PrettyState(s.clone())).expect("final node does not exist")
            })
            .collect();
        Self { graph, start, fin }
    }

    pub fn from_slice(transitions: &[(&str, Transition, &str)], start: &str, fin: &[&str]) -> Self {
        let transitions: Vec<(State, Transition, State)> = transitions
            .into_iter()
            .map(|(src, t, dest)| (set![String::from(*src)], *t, set!(String::from(*dest))))
            .collect();
        let start = set!(String::from(start));
        let fin = fin.into_iter().map(|f| set!(String::from(*f))).collect();
        Self::new(&transitions, start, fin)
    }

    /// subset construction
    pub fn to_dfa(&self) -> Self {
        let start_state: NodeGroup = self.epsilon_closure(&set![self.start.clone()]);
        let mut process_set: HashSet<NodeGroup> = HashSet::from_iter(vec![start_state.clone()]);
        let mut states: HashSet<NodeGroup> = HashSet::new();
        let mut transitions: Vec<(State, Transition, State)> = vec![];

        while !process_set.is_empty() {
            let cur: NodeGroup = process_set.iter().next().unwrap().clone();
            if states.contains(&cur) {
                process_set.remove(&cur);
                continue;
            }
            let mut symbols = self.next_moves(&cur);
            symbols.retain(|x| *x != EPSILON);
            for symbol in symbols {
                let target_state: NodeGroup =
                    self.epsilon_closure(&self.reachable_states(&cur, symbol));
                // println!("{:?} - {:?} ->, target state = {:?}", self.parse_states(&cur), symbol, self.parse_states(&target_state));
                process_set.insert(target_state.clone());
                transitions.push((
                    self.parse_states(&cur),
                    symbol,
                    self.parse_states(&target_state),
                ))
            }
            states.insert(process_set.take(&cur).unwrap());
        }
        let mut final_states: BTreeSet<State> = set![];
        for s in states {
            if self.fin.par_iter().any(|f| s.contains(f)) {
                final_states.insert(self.parse_states(&s));
            }
        }
        Self::new(&transitions, self.parse_states(&start_state), final_states)
    }

    pub fn is_dfa(&self) -> bool {
        let nodes = self.graph.node_indices();
        let dup = nodes.into_iter().any(|node| {
            let mut dup_check = HashSet::new();
            let transitions = self.graph.edges_directed(node, Outgoing);
            transitions
                .into_iter()
                .any(|t| t.weight().0 == EPSILON || dup_check.insert(t.weight()))
        });
        !dup
    }

    pub fn next_moves(&self, src: &NodeGroup) -> Vec<char> {
        src.into_iter()
            .map(|from| {
                // for each node, find the outgoing targets
                self.graph
                    .neighbors_directed(*from, Outgoing)
                    .into_iter()
                    // for each target, map edge to weight
                    .filter_map(|dest| self.graph.find_edge(*from, dest))
                    .filter_map(|edge| self.graph.edge_weight(edge))
                    .map(|ch| ch.0)
            })
            .flatten()
            .collect()
    }

    pub fn epsilon_closure(&self, src: &NodeGroup) -> NodeGroup {
        let mut result = src.clone();
        loop {
            let prev_len = result.len();
            result.extend(self.reachable_states(&result, EPSILON));
            if result.len() == prev_len {
                break;
            }
        }
        result
    }

    /// given set of source states, returns a set of target states by specified transition
    pub fn reachable_states(&self, src: &NodeGroup, transition: Transition) -> NodeGroup {
        src.iter()
            .map(|from| {
                // for each src
                self.graph
                    .neighbors_directed(*from, Outgoing)
                    // filter those destination, collect those having any transition
                    .filter(|dest| {
                        self.graph
                            .edges_connecting(*from, *dest)
                            .any(|edge| edge.weight().0 == transition)
                    })
            })
            .flatten()
            .collect()
    }

    pub fn export(&self, name: &str) -> io::Result<()> {
        // let dot = format!("{:?}", Dot::new(&self.graph));
        let dot = format!("{:?}", 
            Dot::with_attr_getters(&self.graph, &[], &|g, e| String::new(), &|g, n| {
                if n.1.0 == g[self.start].0 {
                    return String::from("shape = triangle");
                }
                if self.fin.contains(&n.0) {
                    return String::from("shape = doublecircle");
                }
                String::new()
            })
        );
        let caller_path = std::env::current_dir()?;
        let dot_path = caller_path.join(format!("{}.dot", name));
        let svg_path = caller_path.join(format!("{}.svg", name));

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(format!("{}.dot", name))?;
        file.write_all(dot.as_bytes());
        file.flush();

        Command::new("dot")
            .arg("-Tsvg")
            .arg(dot_path)
            .arg("-o")
            .arg(svg_path)
            .output()?;
        Ok(())
    }


    /// given set of node indices, extract the states then merge as one state
    fn parse_states(&self, idx: &NodeGroup) -> State {
        idx.iter()
            .map(|i| self.graph[*i].clone().0)
            .flatten()
            .collect()
    }

    fn node_idx<T: PartialEq, E>(graph: &Graph<T, E>, element: &T) -> Option<NodeIndex> {
        graph.node_indices().find(|idx| graph[*idx] == *element)
    }

    /// do not quite understand why petgraph crate does not provide this
    fn get_or_create<T: PartialEq, E>(graph: &mut Graph<T, E>, element: T) -> NodeIndex {
        let exist = Self::node_idx(graph, &element);
        match exist {
            Some(idx) => idx,
            None => graph.add_node(element),
        }
    }

    fn add_edge_no_dup<T: PartialEq, E: PartialEq>(graph: &mut Graph<T, E>, src: T, t: E, dest: T) {
        let src = Self::get_or_create(graph, src);
        let dest = Self::get_or_create(graph, dest);
        let dup = graph
            .edges_connecting(src, dest)
            .any(|edge| *edge.weight() == t);
        if dup {
            return;
        }
        graph.add_edge(src, dest, t);
    }
}
