#[macro_export]
macro_rules! set {
    ($($x:expr),*) => {{
        let mut set = BTreeSet::new();
        $(
            set.insert($x);
        )*
        set
    }};
}
