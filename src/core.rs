use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use anyhow::Result;

use crate::hypergraph::HyperGraph;
use crate::neural::SpikeProcessor;
use crate::storage::StorageEngine;

/// Main database instance for Neurographite
/// 
/// Combines hypergraph data structure with neuromorphic processing
/// to create a database that learns and adapts relationship strengths
/// over time through spiking neural network patterns.
pub struct Database {
    /// The core hypergraph data structure
    graph: Arc<RwLock<HyperGraph>>,
    
    /// Neural processing engine for spike propagation
    neural: Arc<SpikeProcessor>,
    
    /// Storage engine for persistence
    storage: Arc<StorageEngine>,
    
    /// Database configuration
    config: DatabaseConfig,
}

#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub data_dir: String,
    pub spike_threshold: f64,
    pub decay_rate: f64,
    pub refractory_period: u64, // milliseconds
    pub max_cascade_depth: usize,
    pub sync_interval: u64, // seconds
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            data_dir: "./data".to_string(),
            spike_threshold: 0.7,
            decay_rate: 0.99,
            refractory_period: 100,
            max_cascade_depth: 10,
            sync_interval: 60,
        }
    }
}

impl Database {
    /// Create a new Neurographite database instance
    pub async fn new<P: AsRef<Path>>(data_dir: P) -> Result<Self> {
        let config = DatabaseConfig {
            data_dir: data_dir.as_ref().to_string_lossy().to_string(),
            ..Default::default()
        };
        
        let storage = Arc::new(StorageEngine::new(&config.data_dir).await?);
        let graph = Arc::new(RwLock::new(HyperGraph::new()));
        let neural = Arc::new(SpikeProcessor::new(config.clone()));
        
        let db = Self {
            graph,
            neural,
            storage,
            config,
        };
        
        // Load existing data if available
        db.load_from_storage().await?;
        
        Ok(db)
    }
    
    /// Create a new database with custom configuration
    pub async fn with_config(config: DatabaseConfig) -> Result<Self> {
        let storage = Arc::new(StorageEngine::new(&config.data_dir).await?);
        let graph = Arc::new(RwLock::new(HyperGraph::new()));
        let neural = Arc::new(SpikeProcessor::new(config.clone()));
        
        let db = Self {
            graph,
            neural,
            storage,
            config,
        };
        
        db.load_from_storage().await?;
        Ok(db)
    }
    
    /// Add a new node to the hypergraph
    pub async fn add_node(&self, data: serde_json::Value) -> Result<Uuid> {
        let node_id = Uuid::new_v4();
        
        {
            let mut graph = self.graph.write().await;
            graph.add_node(node_id, data)?;
        }
        
        // Trigger neural processing
        self.neural.process_new_node(node_id).await?;
        
        // Persist changes
        self.sync_to_storage().await?;
        
        Ok(node_id)
    }
    
    /// Create a hyperedge connecting multiple nodes
    pub async fn connect_nodes(&self, node_ids: Vec<Uuid>, relationship: String, strength: f64) -> Result<Uuid> {
        let edge_id = Uuid::new_v4();
        
        {
            let mut graph = self.graph.write().await;
            graph.add_hyperedge(edge_id, node_ids.clone(), relationship, strength)?;
        }
        
        // Trigger spike propagation through the new connection
        self.neural.propagate_spike(node_ids, strength).await?;
        
        self.sync_to_storage().await?;
        
        Ok(edge_id)
    }
    
    /// Find similar nodes using neural activation patterns
    pub async fn find_similar(&self, node_id: Uuid, threshold: f64) -> Result<Vec<(Uuid, f64)>> {
        let graph = self.graph.read().await;
        self.neural.find_similar_nodes(&*graph, node_id, threshold).await
    }
    
    /// Discover potential relationships using stable matching algorithm
    pub async fn discover_relationships(&self, max_results: usize) -> Result<Vec<(Uuid, Uuid, f64)>> {
        let graph = self.graph.read().await;
        self.neural.stable_matching(&*graph, max_results).await
    }
    
    /// Get network effects from a node activation
    pub async fn simulate_network_effect(&self, node_id: Uuid, activation_strength: f64) -> Result<Vec<(Uuid, f64)>> {
        let graph = self.graph.read().await;
        self.neural.simulate_cascade(&*graph, node_id, activation_strength).await
    }
    
    /// Load database state from storage
    async fn load_from_storage(&self) -> Result<()> {
        if let Ok(graph_data) = self.storage.load_graph().await {
            let mut graph = self.graph.write().await;
            *graph = graph_data;
        }
        Ok(())
    }
    
    /// Sync current state to storage
    async fn sync_to_storage(&self) -> Result<()> {
        let graph = self.graph.read().await;
        self.storage.save_graph(&*graph).await
    }
    
    /// Get database statistics
    pub async fn stats(&self) -> DatabaseStats {
        let graph = self.graph.read().await;
        let neural_stats = self.neural.stats().await;
        
        DatabaseStats {
            node_count: graph.node_count(),
            edge_count: graph.edge_count(),
            total_spikes: neural_stats.total_spikes,
            active_neurons: neural_stats.active_neurons,
            average_activation: neural_stats.average_activation,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub total_spikes: u64,
    pub active_neurons: usize,
    pub average_activation: f64,
}