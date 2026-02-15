use criterion::{black_box, criterion_group, criterion_main, Criterion};
use neurographite::{Database, HyperGraph};
use tokio::runtime::Runtime;
use uuid::Uuid;

fn bench_spike_propagation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("spike_propagation_small_network", |b| {
        b.iter(|| {
            rt.block_on(async {
                let db = Database::new("./bench_data").await.unwrap();
                
                // Create a small network
                let mut node_ids = Vec::new();
                for i in 0..10 {
                    let data = serde_json::json!({"id": i, "type": "test_node"});
                    let node_id = db.add_node(data).await.unwrap();
                    node_ids.push(node_id);
                }
                
                // Connect nodes in a chain
                for i in 0..9 {
                    db.connect_nodes(
                        vec![node_ids[i], node_ids[i+1]], 
                        "chain_link".to_string(), 
                        0.8
                    ).await.unwrap();
                }
                
                // Benchmark spike propagation
                let effects = black_box(
                    db.simulate_network_effect(node_ids[0], 1.0).await.unwrap()
                );
                
                assert!(!effects.is_empty());
            })
        })
    });
    
    c.bench_function("spike_propagation_large_network", |b| {
        b.iter(|| {
            rt.block_on(async {
                let db = Database::new("./bench_data").await.unwrap();
                
                // Create a larger network
                let mut node_ids = Vec::new();
                for i in 0..100 {
                    let data = serde_json::json!({"id": i, "type": "test_node"});
                    let node_id = db.add_node(data).await.unwrap();
                    node_ids.push(node_id);
                }
                
                // Create a more complex topology (small world)
                for i in 0..100 {
                    // Local connections
                    if i < 99 {
                        db.connect_nodes(
                            vec![node_ids[i], node_ids[i+1]], 
                            "local".to_string(), 
                            0.7
                        ).await.unwrap();
                    }
                    
                    // Random long-distance connections
                    if i % 10 == 0 && i + 20 < 100 {
                        db.connect_nodes(
                            vec![node_ids[i], node_ids[i+20]], 
                            "long_distance".to_string(), 
                            0.4
                        ).await.unwrap();
                    }
                }
                
                // Benchmark spike propagation
                let effects = black_box(
                    db.simulate_network_effect(node_ids[0], 1.0).await.unwrap()
                );
                
                assert!(!effects.is_empty());
            })
        })
    });
}

fn bench_similarity_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("similarity_search", |b| {
        b.iter(|| {
            rt.block_on(async {
                let db = Database::new("./bench_data").await.unwrap();
                
                // Create nodes with varying similarity
                let mut node_ids = Vec::new();
                for i in 0..50 {
                    let data = serde_json::json!({
                        "id": i,
                        "category": i % 5, // 5 categories for similarity
                        "value": i as f64 / 10.0
                    });
                    let node_id = db.add_node(data).await.unwrap();
                    node_ids.push(node_id);
                }
                
                // Add some connections
                for i in 0..25 {
                    db.connect_nodes(
                        vec![node_ids[i], node_ids[i + 25]], 
                        "cross_connect".to_string(), 
                        0.6
                    ).await.unwrap();
                }
                
                // Benchmark similarity search
                let similar = black_box(
                    db.find_similar(node_ids[0], 0.3).await.unwrap()
                );
                
                assert!(!similar.is_empty());
            })
        })
    });
}

criterion_group!(benches, bench_spike_propagation, bench_similarity_search);
criterion_main!(benches);