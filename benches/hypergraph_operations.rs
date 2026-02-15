use criterion::{black_box, criterion_group, criterion_main, Criterion};
use neurographite::HyperGraph;
use uuid::Uuid;

fn bench_node_operations(c: &mut Criterion) {
    c.bench_function("add_nodes", |b| {
        b.iter(|| {
            let mut graph = HyperGraph::new();
            
            for i in 0..1000 {
                let data = serde_json::json!({"id": i, "value": i * 2});
                let node_id = Uuid::new_v4();
                black_box(graph.add_node(node_id, data).unwrap());
            }
            
            assert_eq!(graph.node_count(), 1000);
        })
    });
    
    c.bench_function("lookup_nodes", |b| {
        // Setup
        let mut graph = HyperGraph::new();
        let mut node_ids = Vec::new();
        
        for i in 0..1000 {
            let data = serde_json::json!({"id": i, "value": i * 2});
            let node_id = Uuid::new_v4();
            graph.add_node(node_id, data).unwrap();
            node_ids.push(node_id);
        }
        
        b.iter(|| {
            for &node_id in &node_ids {
                black_box(graph.get_node(&node_id));
            }
        })
    });
}

fn bench_edge_operations(c: &mut Criterion) {
    c.bench_function("add_hyperedges", |b| {
        b.iter(|| {
            let mut graph = HyperGraph::new();
            let mut node_ids = Vec::new();
            
            // Add nodes first
            for i in 0..100 {
                let data = serde_json::json!({"id": i});
                let node_id = Uuid::new_v4();
                graph.add_node(node_id, data).unwrap();
                node_ids.push(node_id);
            }
            
            // Add hyperedges connecting multiple nodes
            for i in 0..50 {
                let edge_id = Uuid::new_v4();
                let connected_nodes = vec![
                    node_ids[i],
                    node_ids[i + 25],
                    node_ids[i + 50],
                ];
                
                black_box(graph.add_hyperedge(
                    edge_id,
                    connected_nodes,
                    format!("relationship_{}", i),
                    0.5 + (i as f64 / 100.0)
                ).unwrap());
            }
            
            assert_eq!(graph.edge_count(), 50);
        })
    });
    
    c.bench_function("neighbor_lookup", |b| {
        // Setup a connected graph
        let mut graph = HyperGraph::new();
        let mut node_ids = Vec::new();
        
        for i in 0..100 {
            let data = serde_json::json!({"id": i});
            let node_id = Uuid::new_v4();
            graph.add_node(node_id, data).unwrap();
            node_ids.push(node_id);
        }
        
        // Create connections
        for i in 0..50 {
            let edge_id = Uuid::new_v4();
            graph.add_hyperedge(
                edge_id,
                vec![node_ids[i], node_ids[i + 50]],
                "connection".to_string(),
                0.7
            ).unwrap();
        }
        
        b.iter(|| {
            for &node_id in &node_ids {
                black_box(graph.get_neighbors(&node_id));
            }
        })
    });
}

fn bench_graph_analysis(c: &mut Criterion) {
    c.bench_function("property_search", |b| {
        // Setup
        let mut graph = HyperGraph::new();
        
        for i in 0..1000 {
            let mut data = serde_json::json!({"id": i, "category": i % 10});
            let node_id = Uuid::new_v4();
            graph.add_node(node_id, data).unwrap();
            
            // Add property to node
            if let Some(node) = graph.get_node_mut(&node_id) {
                node.properties.insert(
                    "searchable".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(i % 5))
                );
            }
        }
        
        b.iter(|| {
            let search_value = serde_json::Value::Number(serde_json::Number::from(2));
            let results = black_box(
                graph.find_nodes_by_property("searchable", &search_value)
            );
            assert!(!results.is_empty());
        })
    });
    
    c.bench_function("activation_decay", |b| {
        // Setup activated nodes
        let mut graph = HyperGraph::new();
        let mut node_ids = Vec::new();
        
        for i in 0..1000 {
            let data = serde_json::json!({"id": i});
            let node_id = Uuid::new_v4();
            graph.add_node(node_id, data).unwrap();
            node_ids.push(node_id);
            
            // Set random activation
            graph.update_node_activation(&node_id, (i as f64) / 1000.0).unwrap();
        }
        
        b.iter(|| {
            black_box(graph.apply_decay(0.99));
        })
    });
}

criterion_group!(benches, bench_node_operations, bench_edge_operations, bench_graph_analysis);
criterion_main!(benches);