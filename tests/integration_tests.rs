use neurographite::{Database, HyperGraph, DatabaseConfig};
use uuid::Uuid;
use tokio;

#[tokio::test]
async fn test_database_creation() {
    let db = Database::new("./test_data").await.unwrap();
    let stats = db.stats().await;
    
    assert_eq!(stats.node_count, 0);
    assert_eq!(stats.edge_count, 0);
}

#[tokio::test]
async fn test_node_operations() {
    let db = Database::new("./test_data").await.unwrap();
    
    // Add a node
    let data = serde_json::json!({
        "name": "Test Node",
        "type": "person",
        "goals": ["investment", "technology"]
    });
    
    let node_id = db.add_node(data.clone()).await.unwrap();
    
    // Verify node was added
    let stats = db.stats().await;
    assert_eq!(stats.node_count, 1);
    
    // The node_id should be a valid UUID
    assert_ne!(node_id, Uuid::nil());
}

#[tokio::test]
async fn test_hyperedge_creation() {
    let db = Database::new("./test_data").await.unwrap();
    
    // Create some nodes
    let node1 = db.add_node(serde_json::json!({"name": "Alice"})).await.unwrap();
    let node2 = db.add_node(serde_json::json!({"name": "Bob"})).await.unwrap();
    let node3 = db.add_node(serde_json::json!({"name": "Charlie"})).await.unwrap();
    
    // Create a hyperedge connecting all three
    let edge_id = db.connect_nodes(
        vec![node1, node2, node3],
        "collaboration".to_string(),
        0.8
    ).await.unwrap();
    
    let stats = db.stats().await;
    assert_eq!(stats.node_count, 3);
    assert_eq!(stats.edge_count, 1);
    assert_ne!(edge_id, Uuid::nil());
}

#[tokio::test]
async fn test_similarity_search() {
    let db = Database::new("./test_data").await.unwrap();
    
    // Create similar nodes
    let similar_data = serde_json::json!({
        "type": "investor", 
        "interests": ["AI", "blockchain"],
        "risk_tolerance": "medium"
    });
    
    let different_data = serde_json::json!({
        "type": "developer",
        "interests": ["gaming", "mobile"],
        "risk_tolerance": "high"
    });
    
    let target_node = db.add_node(similar_data.clone()).await.unwrap();
    let similar_node = db.add_node(similar_data).await.unwrap();
    let different_node = db.add_node(different_data).await.unwrap();
    
    // Add some connections to create structural similarity
    db.connect_nodes(
        vec![target_node, similar_node],
        "shared_interest".to_string(),
        0.7
    ).await.unwrap();
    
    // Search for similar nodes
    let similar_nodes = db.find_similar(target_node, 0.3).await.unwrap();
    
    // Should find at least the similar node
    assert!(!similar_nodes.is_empty());
    assert!(similar_nodes.iter().any(|(id, _)| *id == similar_node));
}

#[tokio::test]
async fn test_network_effect_simulation() {
    let db = Database::new("./test_data").await.unwrap();
    
    // Create a chain of connected nodes
    let mut nodes = Vec::new();
    for i in 0..5 {
        let data = serde_json::json!({"id": i, "value": i * 10});
        let node_id = db.add_node(data).await.unwrap();
        nodes.push(node_id);
    }
    
    // Connect them in a chain
    for i in 0..4 {
        db.connect_nodes(
            vec![nodes[i], nodes[i+1]],
            "chain_link".to_string(),
            0.8
        ).await.unwrap();
    }
    
    // Simulate network effect from the first node
    let effects = db.simulate_network_effect(nodes[0], 1.0).await.unwrap();
    
    // Should propagate to other nodes in the chain
    assert!(!effects.is_empty());
    assert!(effects.iter().any(|(id, _)| *id == nodes[1]));
}

#[tokio::test]
async fn test_relationship_discovery() {
    let db = Database::new("./test_data").await.unwrap();
    
    // Create complementary nodes
    let investor_data = serde_json::json!({
        "type": "investor",
        "capital": 100000,
        "seeking": "AI_startups"
    });
    
    let startup_data = serde_json::json!({
        "type": "startup",
        "needs": "capital",
        "domain": "AI_startups"
    });
    
    let investor_node = db.add_node(investor_data).await.unwrap();
    let startup_node = db.add_node(startup_data).await.unwrap();
    
    // Trigger some neural activity to make them "active"
    db.connect_nodes(vec![investor_node], "self_activation".to_string(), 0.8).await.unwrap();
    db.connect_nodes(vec![startup_node], "self_activation".to_string(), 0.8).await.unwrap();
    
    // Discover relationships
    let relationships = db.discover_relationships(5).await.unwrap();
    
    // Should find the complementary relationship
    assert!(!relationships.is_empty());
}

#[tokio::test]
async fn test_custom_configuration() {
    let config = DatabaseConfig {
        data_dir: "./test_custom_data".to_string(),
        spike_threshold: 0.5,
        decay_rate: 0.95,
        refractory_period: 50,
        max_cascade_depth: 5,
        sync_interval: 30,
    };
    
    let db = Database::with_config(config.clone()).await.unwrap();
    
    // Test that custom config affects behavior
    let node_id = db.add_node(serde_json::json!({"test": true})).await.unwrap();
    
    // Should be able to create network effects with custom depth
    let effects = db.simulate_network_effect(node_id, 0.6).await.unwrap();
    
    // Basic functionality should work with custom config
    assert!(effects.is_empty()); // No connections = no effects
}

#[test]
fn test_hypergraph_direct_operations() {
    let mut graph = HyperGraph::new();
    
    // Test adding nodes
    let node1 = Uuid::new_v4();
    let node2 = Uuid::new_v4();
    
    graph.add_node(node1, serde_json::json!({"name": "Node1"})).unwrap();
    graph.add_node(node2, serde_json::json!({"name": "Node2"})).unwrap();
    
    assert_eq!(graph.node_count(), 2);
    
    // Test adding hyperedge
    let edge_id = Uuid::new_v4();
    graph.add_hyperedge(
        edge_id,
        vec![node1, node2],
        "test_relationship".to_string(),
        0.7
    ).unwrap();
    
    assert_eq!(graph.edge_count(), 1);
    
    // Test neighbor lookup
    let neighbors = graph.get_neighbors(&node1);
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0].id, node2);
    
    // Test activation update
    graph.update_node_activation(&node1, 0.8).unwrap();
    let node = graph.get_node(&node1).unwrap();
    assert_eq!(node.activation_level, 0.8);
    assert_eq!(node.spike_count, 1); // Should spike at 0.8 > 0.7 threshold
    
    // Test decay
    graph.apply_decay(0.9);
    let node = graph.get_node(&node1).unwrap();
    assert!((node.activation_level - 0.72).abs() < 0.001); // 0.8 * 0.9 = 0.72
}

#[tokio::test]
async fn test_error_handling() {
    let db = Database::new("./test_data").await.unwrap();
    
    // Test connecting non-existent nodes
    let fake_node1 = Uuid::new_v4();
    let fake_node2 = Uuid::new_v4();
    
    let result = db.connect_nodes(
        vec![fake_node1, fake_node2],
        "fake_connection".to_string(),
        0.5
    ).await;
    
    assert!(result.is_err());
    
    // Test similarity search on non-existent node
    let similarity_result = db.find_similar(fake_node1, 0.5).await;
    
    // Should handle gracefully (may return empty results)
    assert!(similarity_result.is_ok());
}