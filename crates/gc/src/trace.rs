use std::collections::HashSet;

pub trait Trace {
    fn trace(&self, visited: &mut HashSet<usize>);
}

impl<T: Trace> Trace for Vec<T> {
    fn trace(&self, visited: &mut HashSet<usize>) {
        for item in self {
            item.trace(visited);
        }
    }
}

impl<K, V: Trace, S> Trace for std::collections::HashMap<K, V, S> {
    fn trace(&self, visited: &mut HashSet<usize>) {
        for val in self.values() {
            val.trace(visited);
        }
    }
}
