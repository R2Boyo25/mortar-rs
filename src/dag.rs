use std::collections::HashMap;

/// A [Directed Acyclic Graph](https://en.wikipedia.org/wiki/Directed_acyclic_graph).
///
/// # Examples
///
/// ```
/// let mut graph = mortar::dag::DAG::new();
///
/// graph.add_node("a".to_owned(), None);
/// graph.add_node("b".to_owned(), None);
/// graph.add_node("c".to_owned(), Some(vec!["a".to_owned()]));
/// graph.add_node("d".to_owned(), None);
/// graph.add_dep("d".to_owned(), "b".to_owned());
///
/// assert_eq!(graph.transitive_reduction().iter().map(|x| {let mut y = x.clone(); y.sort(); y}).collect::<Vec<_>>(), vec![vec!["a".to_owned(), "b".to_owned()], vec!["c".to_owned(), "d".to_owned()]]);
/// ```
pub struct DAG {
    graph: HashMap<String, Vec<String>>,
}

impl DAG {
    /// Creates a new DAG.
    pub fn new() -> Self {
        DAG {
            graph: HashMap::new(),
        }
    }

    /// Adds a new node to the DAG.
    pub fn add_node(&mut self, name: String, deps: Option<Vec<String>>) {
        self.graph.insert(name, deps.unwrap_or(vec![]));
    }

    /// Adds a new dependency to the DAG.
    pub fn add_dep(&mut self, name: String, dep: String) {
        self.graph
            .get_mut(&name)
            .expect(&format!("Node \"{}\" does not exist", name))
            .push(dep);
    }

    /// Gets the dependencies of a node.
    pub fn deps(&self, name: String) -> Vec<String> {
        self.graph[&name].clone()
    }

    /// Gets the nodes that depend on a node.
    pub fn reverse_deps(&self, name: String) -> Vec<String> {
        let mut rdeps: Vec<String> = vec![];

        for (node_name, deps) in self.graph.iter() {
            if deps.contains(&name) {
                rdeps.push(node_name.to_owned());
            }
        }

        rdeps
    }

    /// Generates the [transitive reduction](https://en.wikipedia.org/wiki/Directed_acyclic_graph#Reachability_relation.2C_transitive_closure.2C_and_transitive_reduction) of the DAG.
    pub fn transitive_reduction(&self) -> Vec<Vec<String>> {
        let mut tr = vec![self
            .graph
            .keys()
            .map(|x| x.to_owned())
            .collect::<Vec<String>>()];
        let mut temp_tr = tr.clone();

        let mut changed_last = false;
        let mut changed = false;

        loop {
            for (id, layer) in tr.iter().enumerate() {
                for item in layer {
                    if self.deps(item.to_owned()).iter().any(|x| layer.contains(x)) {
                        changed = true;

                        if id == temp_tr.len() - 1 {
                            temp_tr.push(vec![item.to_owned()]);
                        } else {
                            temp_tr[id + 1].push(item.to_owned());
                        }

                        let pos = temp_tr[id].iter().position(|x| x == item).unwrap();

                        temp_tr[id].remove(pos);
                    }
                }
            }

            tr = temp_tr.clone();

            if !changed_last && changed {
                break;
            }

            changed_last = changed;
        }

        tr
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn transitive_reduction() {
        let mut graph = crate::dag::DAG::new();

        graph.add_node("a".to_string(), None);
        graph.add_node("b".to_string(), Some(vec!["a".to_string()]));

        assert_eq!(
            graph.transitive_reduction(),
            vec![vec!["a".to_string()], vec!["b".to_string()]]
        );
    }
}
