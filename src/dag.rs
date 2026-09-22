use crate::model::PipelineDag;
use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Eq)]
pub struct DagValidationResult {
    pub is_valid: bool,
    pub execution_order: Vec<String>,
    pub errors: Vec<String>,
}

/// Memvalidasi DAG dan menghitung urutan eksekusi topologis secara deterministik
pub fn validate_and_sort_dag(dag: &PipelineDag) -> DagValidationResult {
    let mut errors = Vec::new();

    if dag.nodes.is_empty() {
        return DagValidationResult {
            is_valid: false,
            execution_order: Vec::new(),
            errors: vec!["Pipeline DAG tidak memiliki simpul modul (nodes kosong)".to_string()],
        };
    }

    let node_set: HashSet<&String> = dag.nodes.iter().collect();
    let mut graph = DiGraph::<&str, ()>::new();
    let mut node_indices = HashMap::new();

    for node in &dag.nodes {
        let idx = graph.add_node(node.as_str());
        node_indices.insert(node.as_str(), idx);
    }

    for (idx, edge) in dag.edges.iter().enumerate() {
        if !node_set.contains(&edge.from) {
            errors.push(format!("Edge #{idx}: simpul asal '{}' tidak ditemukan di nodes", edge.from));
        }
        if !node_set.contains(&edge.to) {
            errors.push(format!("Edge #{idx}: simpul tujuan '{}' tidak ditemukan di nodes", edge.to));
        }
        if edge.from == edge.to {
            errors.push(format!("Edge #{idx}: mendeteksi self-loop pada simpul '{}'", edge.from));
        }

        if let (Some(&from_idx), Some(&to_idx)) = (node_indices.get(edge.from.as_str()), node_indices.get(edge.to.as_str())) {
            graph.add_edge(from_idx, to_idx, ());
        }
    }

    if !errors.is_empty() {
        return DagValidationResult {
            is_valid: false,
            execution_order: Vec::new(),
            errors,
        };
    }

    // Topological sort dengan deteksi siklus via Petgraph
    match toposort(&graph, None) {
        Ok(sorted_indices) => {
            let execution_order = sorted_indices
                .into_iter()
                .map(|idx| graph[idx].to_string())
                .collect();

            DagValidationResult {
                is_valid: true,
                execution_order,
                errors: Vec::new(),
            }
        }
        Err(cycle) => {
            let cycle_node = graph[cycle.node_id()];
            DagValidationResult {
                is_valid: false,
                execution_order: Vec::new(),
                errors: vec![format!("Mendeteksi dependensi melingkar (deadlock cycle) pada simpul: '{cycle_node}'")],
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DagEdge;

    #[test]
    fn test_valid_dag_linear() {
        let dag = PipelineDag {
            nodes: vec!["a".into(), "b".into(), "c".into()],
            edges: vec![
                DagEdge { from: "a".into(), to: "b".into() },
                DagEdge { from: "b".into(), to: "c".into() },
            ],
        };
        let res = validate_and_sort_dag(&dag);
        assert!(res.is_valid);
        assert_eq!(res.execution_order, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_cycle_detection() {
        let dag = PipelineDag {
            nodes: vec!["x".into(), "y".into()],
            edges: vec![
                DagEdge { from: "x".into(), to: "y".into() },
                DagEdge { from: "y".into(), to: "x".into() },
            ],
        };
        let res = validate_and_sort_dag(&dag);
        assert!(!res.is_valid);
        assert!(res.errors[0].contains("dependensi melingkar"));
    }
}
