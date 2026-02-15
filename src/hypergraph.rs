use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow::{Result, anyhow};

/// A hypergraph where nodes can be connected by hyperedges
/// that link multiple nodes simultaneously (not just pairs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperGraph {
    nodes: HashMap<Uuid, HyperNode>,
    edges: HashMap<Uuid, HyperEdge>,
    
    // Adjacency information for efficient traversal
    node_to_edges: HashMap<Uuid, HashSet<Uuid>>,
    edge_to_nodes: HashMap<Uuid, Vec<Uuid>>,
}

/// A node in the hypergraph representing an entity (person, goal, resource)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperNode {
    pub id: Uuid,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    // Neural properties
    pub activation_level: f64,
    pub last_spike_time: Option<DateTime<Utc>>,
    pub spike_count: u64,
    
    // Metadata
    pub node_type: String,
    pub tags: Vec<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

/// A hyperedge connecting multiple nodes with a relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperEdge {
    pub id: Uuid,
    pub node_ids: Vec<Uuid>,
    pub relationship: String,
    pub strength: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    // Neural properties  
    pub conductance: f64,
    pub last_activation: Option<DateTime<Utc>>,
    pub activation_count: u64,
    
    // Metadata
    pub edge_type: EdgeType,
    pub weight_decay: f64,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    /// Bidirectional relationship (mutual benefit)
    Symmetric,
    /// Directional relationship (one-way influence)
    Directional { from: Uuid, to: Vec<Uuid> },
    /// Hub relationship (one central node, many peripheral)
    Hub { center: Uuid, periphery: Vec<Uuid> },
    /// Chain relationship (sequential dependencies)
    Chain,
}

impl HyperGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            node_to_edges: HashMap::new(),
            edge_to_nodes: HashMap::new(),
        }
    }
    
    /// Add a new node to the hypergraph
    pub fn add_node(&mut self, id: Uuid, data: serde_json::Value) -> Result<()> {
        let now = Utc::now();
        
        let node = HyperNode {
            id,
            data,
            created_at: now,
            updated_at: now,
            activation_level: 0.0,
            last_spike_time: None,
            spike_count: 0,
            node_type: "generic".to_string(),
            tags: Vec::new(),
            properties: HashMap::new(),
        };
        
        self.nodes.insert(id, node);
        self.node_to_edges.insert(id, HashSet::new());
        
        Ok(())
    }
    
    /// Add a hyperedge connecting multiple nodes
    pub fn add_hyperedge(
        &mut self,
        id: Uuid,
        node_ids: Vec<Uuid>,
        relationship: String,
        strength: f64,
    ) -> Result<()> {
        // Validate all nodes exist
        for node_id in &node_ids {
            if !self.nodes.contains_key(node_id) {
                return Err(anyhow!("Node {} not found", node_id));
            }
        }
        
        let now = Utc::now();
        
        let edge = HyperEdge {
            id,
            node_ids: node_ids.clone(),
            relationship,
            strength,
            created_at: now,
            updated_at: now,
            conductance: strength,
            last_activation: None,
            activation_count: 0,
            edge_type: EdgeType::Symmetric,
            weight_decay: 0.99,
            properties: HashMap::new(),
        };
        
        // Update adjacency information
        self.edges.insert(id, edge);
        self.edge_to_nodes.insert(id, node_ids.clone());
        
        for node_id in node_ids {
            self.node_to_edges
                .entry(node_id)
                .or_insert_with(HashSet::new)
                .insert(id);
        }
        
        Ok(())
    }
    
    /// Get a node by ID
    pub fn get_node(&self, id: &Uuid) -> Option<&HyperNode> {
        self.nodes.get(id)
    }
    
    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, id: &Uuid) -> Option<&mut HyperNode> {
        self.nodes.get_mut(id)
    }
    
    /// Get an edge by ID
    pub fn get_edge(&self, id: &Uuid) -> Option<&HyperEdge> {
        self.edges.get(id)
    }
    
    /// Get a mutable edge by ID
    pub fn get_edge_mut(&mut self, id: &Uuid) -> Option<&mut HyperEdge> {
        self.edges.get_mut(id)
    }
    
    /// Get all edges connected to a node
    pub fn get_node_edges(&self, node_id: &Uuid) -> Vec<&HyperEdge> {
        if let Some(edge_ids) = self.node_to_edges.get(node_id) {
            edge_ids
                .iter()
                .filter_map(|edge_id| self.edges.get(edge_id))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get all nodes connected to a node through any edge
    pub fn get_neighbors(&self, node_id: &Uuid) -> Vec<&HyperNode> {
        let mut neighbors = HashSet::new();
        
        if let Some(edge_ids) = self.node_to_edges.get(node_id) {
            for edge_id in edge_ids {
                if let Some(connected_nodes) = self.edge_to_nodes.get(edge_id) {
                    for connected_id in connected_nodes {
                        if connected_id != node_id {
                            neighbors.insert(connected_id);
                        }
                    }
                }
            }
        }
        
        neighbors
            .into_iter()
            .filter_map(|id| self.nodes.get(id))
            .collect()
    }
    
    /// Find nodes by properties
    pub fn find_nodes_by_property(&self, key: &str, value: &serde_json::Value) -> Vec<&HyperNode> {
        self.nodes
            .values()
            .filter(|node| {
                node.properties.get(key).map_or(false, |v| v == value)
            })
            .collect()
    }
    
    /// Update node activation level
    pub fn update_node_activation(&mut self, node_id: &Uuid, activation: f64) -> Result<()> {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.activation_level = activation;
            node.updated_at = Utc::now();
            
            // Record spike if above threshold
            if activation > 0.7 {
                node.last_spike_time = Some(Utc::now());
                node.spike_count += 1;
            }
            
            Ok(())
        } else {
            Err(anyhow!("Node {} not found", node_id))
        }
    }
    
    /// Update edge conductance based on usage
    pub fn update_edge_conductance(&mut self, edge_id: &Uuid, new_conductance: f64) -> Result<()> {
        if let Some(edge) = self.edges.get_mut(edge_id) {
            edge.conductance = new_conductance;
            edge.last_activation = Some(Utc::now());
            edge.activation_count += 1;
            edge.updated_at = Utc::now();
            Ok(())
        } else {
            Err(anyhow!("Edge {} not found", edge_id))
        }
    }
    
    /// Apply decay to all node activations and edge conductances
    pub fn apply_decay(&mut self, decay_rate: f64) {
        for node in self.nodes.values_mut() {
            node.activation_level *= decay_rate;
        }
        
        for edge in self.edges.values_mut() {
            edge.conductance *= edge.weight_decay;
        }
    }
    
    /// Get graph statistics
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
    
    pub fn average_degree(&self) -> f64 {
        if self.nodes.is_empty() {
            return 0.0;
        }
        
        let total_connections: usize = self.node_to_edges.values().map(|edges| edges.len()).sum();
        total_connections as f64 / self.nodes.len() as f64
    }
    
    /// Get all nodes
    pub fn nodes(&self) -> &HashMap<Uuid, HyperNode> {
        &self.nodes
    }
    
    /// Get all edges
    pub fn edges(&self) -> &HashMap<Uuid, HyperEdge> {
        &self.edges
    }
}