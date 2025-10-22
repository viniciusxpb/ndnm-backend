// ndnm-brazil/src/execution/resolver.rs
//
// Resolvedor de dependências do grafo
// Converte grafo de nodes → lista ordenada de execução (depth-first)

use super::types::{WorkflowGraph, GraphNode};
use std::collections::{HashMap, HashSet};

/// Resolvedor de dependências
pub struct DependencyResolver<'a> {
    /// Mapa de node_id → GraphNode (pra busca rápida)
    node_map: HashMap<String, &'a GraphNode>,
    /// Mapa de node_id → lista de dependências (nodes que alimentam ele)
    dependencies: HashMap<String, Vec<String>>,
}

impl<'a> DependencyResolver<'a> {
    /// Cria novo resolver a partir do grafo
    pub fn new(graph: &'a WorkflowGraph) -> Self {
        let mut node_map = HashMap::new();
        let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();

        // Popula node_map
        for node in &graph.nodes {
            node_map.insert(node.id.clone(), node);
            dependencies.insert(node.id.clone(), Vec::new());
        }

        // Popula dependências a partir das conexões
        // Se A → B (A conectado em B), então B depende de A
        for conn in &graph.connections {
            dependencies
                .entry(conn.to_node_id.clone())
                .or_insert_with(Vec::new)
                .push(conn.from_node_id.clone());
        }

        Self {
            node_map,
            dependencies,
        }
    }

    /// Resolve dependências a partir de um node (ex: Play node)
    /// Retorna lista ordenada de nodes pra executar (depth-first)
    ///
    /// Exemplo:
    ///     A
    ///    / \
    ///   B   D
    ///    \ /
    ///     C
    ///     ↓
    ///   Play
    ///
    /// Resultado: [A, B, D, C]
    /// (A primeiro porque B e D dependem dele, depois B e D por ordem de ID, depois C)
    pub fn resolve_from(&self, start_node_id: &str) -> Result<Vec<&'a GraphNode>, String> {
        let mut visited = HashSet::new();
        let mut execution_order = Vec::new();

        self.visit_node(start_node_id, &mut visited, &mut execution_order)?;

        Ok(execution_order)
    }

    /// Visita um node recursivamente (depth-first)
    fn visit_node(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        execution_order: &mut Vec<&'a GraphNode>,
    ) -> Result<(), String> {
        // Se já visitou, pula (evita loops infinitos)
        if visited.contains(node_id) {
            return Ok(());
        }

        // Marca como visitado
        visited.insert(node_id.to_string());

        // Busca o node
        let node = self
            .node_map
            .get(node_id)
            .ok_or_else(|| format!("Node não encontrado: {}", node_id))?;

        // Primeiro, resolve todas as dependências (depth-first!)
        if let Some(deps) = self.dependencies.get(node_id) {
            // Ordena dependências por ID (pra ser determinístico)
            let mut sorted_deps = deps.clone();
            sorted_deps.sort();

            for dep_id in sorted_deps {
                self.visit_node(&dep_id, visited, execution_order)?;
            }
        }

        // Depois de resolver dependências, adiciona o node atual à ordem de execução
        execution_order.push(node);

        Ok(())
    }

    /// Resolve múltiplos nodes de partida (útil se tiver múltiplos Plays)
    #[allow(dead_code)]
    pub fn resolve_from_multiple(&self, start_node_ids: &[String]) -> Result<Vec<&'a GraphNode>, String> {
        let mut visited = HashSet::new();
        let mut execution_order = Vec::new();

        // Ordena por ID pra ser determinístico
        let mut sorted_starts = start_node_ids.to_vec();
        sorted_starts.sort();

        for node_id in sorted_starts {
            self.visit_node(&node_id, &mut visited, &mut execution_order)?;
        }

        Ok(execution_order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::Connection;
    use serde_json::json;

    #[test]
    fn test_simple_chain() {
        // A → B → C
        let graph = WorkflowGraph {
            nodes: vec![
                GraphNode {
                    id: "A".to_string(),
                    node_type: "add".to_string(),
                    port: 3000,
                    label: "Node A".to_string(),
                    data: Default::default(),
                },
                GraphNode {
                    id: "B".to_string(),
                    node_type: "multiply".to_string(),
                    port: 3001,
                    label: "Node B".to_string(),
                    data: Default::default(),
                },
                GraphNode {
                    id: "C".to_string(),
                    node_type: "playButton".to_string(),
                    port: 3020,
                    label: "Play".to_string(),
                    data: Default::default(),
                },
            ],
            connections: vec![
                Connection {
                    from_node_id: "A".to_string(),
                    from_output_index: 0,
                    to_node_id: "B".to_string(),
                    to_input_index: 0,
                },
                Connection {
                    from_node_id: "B".to_string(),
                    from_output_index: 0,
                    to_node_id: "C".to_string(),
                    to_input_index: 0,
                },
            ],
        };

        let resolver = DependencyResolver::new(&graph);
        let order = resolver.resolve_from("C").unwrap();

        // Ordem esperada: A → B → C
        assert_eq!(order.len(), 3);
        assert_eq!(order[0].id, "A");
        assert_eq!(order[1].id, "B");
        assert_eq!(order[2].id, "C");
    }

    #[test]
    fn test_diamond_pattern() {
        //     A
        //    / \
        //   B   D
        //    \ /
        //     C
        let graph = WorkflowGraph {
            nodes: vec![
                GraphNode {
                    id: "A".to_string(),
                    node_type: "add".to_string(),
                    port: 3000,
                    label: "Node A".to_string(),
                    data: Default::default(),
                },
                GraphNode {
                    id: "B".to_string(),
                    node_type: "multiply".to_string(),
                    port: 3001,
                    label: "Node B".to_string(),
                    data: Default::default(),
                },
                GraphNode {
                    id: "D".to_string(),
                    node_type: "subtract".to_string(),
                    port: 3002,
                    label: "Node D".to_string(),
                    data: Default::default(),
                },
                GraphNode {
                    id: "C".to_string(),
                    node_type: "playButton".to_string(),
                    port: 3020,
                    label: "Play".to_string(),
                    data: Default::default(),
                },
            ],
            connections: vec![
                Connection {
                    from_node_id: "A".to_string(),
                    from_output_index: 0,
                    to_node_id: "B".to_string(),
                    to_input_index: 0,
                },
                Connection {
                    from_node_id: "A".to_string(),
                    from_output_index: 1,
                    to_node_id: "D".to_string(),
                    to_input_index: 0,
                },
                Connection {
                    from_node_id: "B".to_string(),
                    from_output_index: 0,
                    to_node_id: "C".to_string(),
                    to_input_index: 0,
                },
                Connection {
                    from_node_id: "D".to_string(),
                    from_output_index: 0,
                    to_node_id: "C".to_string(),
                    to_input_index: 1,
                },
            ],
        };

        let resolver = DependencyResolver::new(&graph);
        let order = resolver.resolve_from("C").unwrap();

        // Ordem esperada: A, depois B e D (ordenados por ID), depois C
        assert_eq!(order.len(), 4);
        assert_eq!(order[0].id, "A");
        // B vem antes de D porque "B" < "D" alfabeticamente
        assert_eq!(order[1].id, "B");
        assert_eq!(order[2].id, "D");
        assert_eq!(order[3].id, "C");
    }
}
