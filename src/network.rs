use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::hypergraph::{HyperGraph, HyperNode};

/// Network effect analysis for DeepRuin goal alignment
#[derive(Debug, Clone)]
pub struct NetworkEffect {
    pub source_node: Uuid,
    pub affected_nodes: Vec<(Uuid, f64)>, // Node ID and effect strength
    pub total_effect: f64,
    pub cascade_depth: usize,
    pub effect_type: EffectType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectType {
    /// Positive network effects (mutual benefit)
    Synergistic,
    /// Competitive effects (zero-sum)
    Competitive, 
    /// Asymmetric effects (one benefits more)
    Asymmetric { primary_beneficiary: Uuid },
    /// Neutral (no significant effect)
    Neutral,
}

/// Goal alignment analysis between entities
#[derive(Debug, Clone)]
pub struct GoalAlignment {
    pub node1: Uuid,
    pub node2: Uuid,
    pub alignment_score: f64,
    pub alignment_type: AlignmentType,
    pub potential_value: f64,
    pub risks: Vec<String>,
    pub opportunities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlignmentType {
    /// Goals are completely aligned
    Perfect,
    /// Goals are mostly aligned with some differences
    High,
    /// Goals are partially aligned
    Moderate,
    /// Goals conflict but could be negotiated
    Conflicting,
    /// Goals are incompatible
    Incompatible,
}

/// Network analyzer for discovering relationships and effects
pub struct NetworkAnalyzer;

impl NetworkAnalyzer {
    /// Analyze network effects from a node activation
    pub fn analyze_network_effects(
        graph: &HyperGraph,
        source_node: Uuid,
        activation_strength: f64,
        max_depth: usize,
    ) -> Result<NetworkEffect> {
        let mut affected_nodes = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = vec![(source_node, activation_strength, 0)];
        let mut total_effect = 0.0;
        let mut max_cascade_depth = 0;
        
        while let Some((current_node, current_strength, depth)) = queue.pop() {
            if depth >= max_depth || visited.contains(&current_node) {
                continue;
            }
            
            visited.insert(current_node);
            max_cascade_depth = max_cascade_depth.max(depth);
            
            if current_node != source_node {
                affected_nodes.push((current_node, current_strength));
                total_effect += current_strength;
            }
            
            // Propagate to neighbors
            let neighbors = graph.get_neighbors(&current_node);
            for neighbor in neighbors {
                if !visited.contains(&neighbor.id) {
                    let edges = graph.get_node_edges(&current_node);
                    let mut propagated_strength = 0.0;
                    
                    // Calculate propagation strength based on edge properties
                    for edge in edges {
                        if edge.node_ids.contains(&neighbor.id) {
                            propagated_strength = propagated_strength.max(
                                current_strength * edge.conductance * 0.9 // 10% decay per hop
                            );
                        }
                    }
                    
                    if propagated_strength > 0.01 {
                        queue.push((neighbor.id, propagated_strength, depth + 1));
                    }
                }
            }
        }
        
        // Determine effect type based on the pattern of effects
        let effect_type = Self::classify_effect_type(&affected_nodes, total_effect);
        
        Ok(NetworkEffect {
            source_node,
            affected_nodes,
            total_effect,
            cascade_depth: max_cascade_depth,
            effect_type,
        })
    }
    
    /// Analyze goal alignment between two nodes
    pub fn analyze_goal_alignment(
        graph: &HyperGraph,
        node1: Uuid,
        node2: Uuid,
    ) -> Result<GoalAlignment> {
        let node1_data = graph.get_node(&node1).ok_or_else(|| anyhow::anyhow!("Node 1 not found"))?;
        let node2_data = graph.get_node(&node2).ok_or_else(|| anyhow::anyhow!("Node 2 not found"))?;
        
        // Analyze structural similarity (shared connections)
        let structural_alignment = Self::calculate_structural_alignment(graph, node1, node2);
        
        // Analyze semantic alignment (data similarity)
        let semantic_alignment = Self::calculate_semantic_alignment(node1_data, node2_data)?;
        
        // Analyze temporal alignment (activity patterns)
        let temporal_alignment = Self::calculate_temporal_alignment(node1_data, node2_data);
        
        // Combine scores with weights
        let alignment_score = (structural_alignment * 0.4) + 
                            (semantic_alignment * 0.4) + 
                            (temporal_alignment * 0.2);
        
        let alignment_type = match alignment_score {
            s if s >= 0.8 => AlignmentType::Perfect,
            s if s >= 0.6 => AlignmentType::High,
            s if s >= 0.4 => AlignmentType::Moderate,
            s if s >= 0.2 => AlignmentType::Conflicting,
            _ => AlignmentType::Incompatible,
        };
        
        // Calculate potential value and identify risks/opportunities
        let potential_value = Self::calculate_potential_value(&alignment_type, alignment_score);
        let (risks, opportunities) = Self::identify_risks_and_opportunities(graph, node1_data, node2_data, &alignment_type);
        
        Ok(GoalAlignment {
            node1,
            node2,
            alignment_score,
            alignment_type,
            potential_value,
            risks,
            opportunities,
        })
    }
    
    /// Find optimal pairs using stable matching algorithm
    pub fn find_optimal_pairs(
        graph: &HyperGraph,
        candidate_nodes: &[Uuid],
        max_pairs: usize,
    ) -> Result<Vec<GoalAlignment>> {
        let mut pairs = Vec::new();
        let mut preferences: HashMap<Uuid, Vec<(Uuid, f64)>> = HashMap::new();
        
        // Calculate preferences for each node
        for &node1 in candidate_nodes {
            let mut node_preferences = Vec::new();
            
            for &node2 in candidate_nodes {
                if node1 != node2 {
                    match Self::analyze_goal_alignment(graph, node1, node2) {
                        Ok(alignment) => {
                            node_preferences.push((node2, alignment.alignment_score));
                        }
                        Err(_) => continue,
                    }
                }
            }
            
            // Sort by alignment score (highest first)
            node_preferences.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            preferences.insert(node1, node_preferences);
        }
        
        // Simple greedy matching (not fully stable, but efficient)
        let mut matched = HashSet::new();
        
        for &node1 in candidate_nodes {
            if matched.contains(&node1) {
                continue;
            }
            
            if let Some(node_prefs) = preferences.get(&node1) {
                for &(node2, score) in node_prefs {
                    if !matched.contains(&node2) && score > 0.3 {
                        // Create the alignment
                        if let Ok(alignment) = Self::analyze_goal_alignment(graph, node1, node2) {
                            pairs.push(alignment);
                            matched.insert(node1);
                            matched.insert(node2);
                            break;
                        }
                    }
                }
            }
            
            if pairs.len() >= max_pairs {
                break;
            }
        }
        
        // Sort pairs by potential value
        pairs.sort_by(|a, b| b.potential_value.partial_cmp(&a.potential_value).unwrap_or(std::cmp::Ordering::Equal));
        pairs.truncate(max_pairs);
        
        Ok(pairs)
    }
    
    /// Calculate centrality measures for nodes
    pub fn calculate_centrality_measures(graph: &HyperGraph) -> HashMap<Uuid, CentralityMeasures> {
        let mut measures = HashMap::new();
        
        for (node_id, _) in graph.nodes() {
            let degree = graph.get_node_edges(node_id).len() as f64;
            let neighbors = graph.get_neighbors(node_id);
            
            // Betweenness centrality (simplified)
            let betweenness = Self::calculate_betweenness_centrality(graph, *node_id);
            
            // Closeness centrality (simplified)
            let closeness = Self::calculate_closeness_centrality(graph, *node_id);
            
            // Eigenvector centrality (simplified)
            let eigenvector = Self::calculate_eigenvector_centrality(graph, *node_id, &neighbors);
            
            measures.insert(*node_id, CentralityMeasures {
                degree,
                betweenness,
                closeness,
                eigenvector,
            });
        }
        
        measures
    }
    
    // Helper methods
    
    fn classify_effect_type(affected_nodes: &[(Uuid, f64)], total_effect: f64) -> EffectType {
        if total_effect > 0.5 && affected_nodes.len() > 2 {
            EffectType::Synergistic
        } else if total_effect < 0.1 {
            EffectType::Neutral
        } else if let Some(&(primary_node, primary_effect)) = affected_nodes.first() {
            if primary_effect > total_effect * 0.7 {
                EffectType::Asymmetric { primary_beneficiary: primary_node }
            } else {
                EffectType::Competitive
            }
        } else {
            EffectType::Neutral
        }
    }
    
    fn calculate_structural_alignment(graph: &HyperGraph, node1: Uuid, node2: Uuid) -> f64 {
        let neighbors1 = graph.get_neighbors(&node1);
        let neighbors2 = graph.get_neighbors(&node2);
        
        if neighbors1.is_empty() && neighbors2.is_empty() {
            return 1.0; // Both isolated - perfectly aligned
        }
        
        let shared_neighbors = neighbors1
            .iter()
            .filter(|n1| neighbors2.iter().any(|n2| n1.id == n2.id))
            .count();
        
        let total_neighbors = neighbors1.len() + neighbors2.len() - shared_neighbors;
        
        if total_neighbors == 0 {
            1.0
        } else {
            shared_neighbors as f64 / total_neighbors as f64
        }
    }
    
    fn calculate_semantic_alignment(node1: &HyperNode, node2: &HyperNode) -> Result<f64> {
        // Simple semantic similarity based on tags and properties
        let mut similarity_factors = Vec::new();
        
        // Tag similarity
        let shared_tags = node1.tags.iter().filter(|tag| node2.tags.contains(tag)).count();
        let total_tags = (node1.tags.len() + node2.tags.len()).max(1);
        let tag_similarity = shared_tags as f64 / total_tags as f64;
        similarity_factors.push(tag_similarity);
        
        // Node type similarity
        let type_similarity = if node1.node_type == node2.node_type { 1.0 } else { 0.0 };
        similarity_factors.push(type_similarity);
        
        // Property similarity (simplified - just count matching keys)
        let shared_props = node1.properties.keys()
            .filter(|key| node2.properties.contains_key(*key))
            .count();
        let total_props = (node1.properties.len() + node2.properties.len()).max(1);
        let prop_similarity = shared_props as f64 / total_props as f64;
        similarity_factors.push(prop_similarity);
        
        // Average the factors
        let avg_similarity = similarity_factors.iter().sum::<f64>() / similarity_factors.len() as f64;
        Ok(avg_similarity)
    }
    
    fn calculate_temporal_alignment(node1: &HyperNode, node2: &HyperNode) -> f64 {
        // Simple temporal alignment based on spike patterns
        let both_active = node1.last_spike_time.is_some() && node2.last_spike_time.is_some();
        let both_inactive = node1.last_spike_time.is_none() && node2.last_spike_time.is_none();
        
        if both_active || both_inactive {
            0.8 // High alignment
        } else {
            0.2 // Low alignment
        }
    }
    
    fn calculate_potential_value(alignment_type: &AlignmentType, score: f64) -> f64 {
        match alignment_type {
            AlignmentType::Perfect => score * 10.0,
            AlignmentType::High => score * 7.0,
            AlignmentType::Moderate => score * 4.0,
            AlignmentType::Conflicting => score * 2.0,
            AlignmentType::Incompatible => 0.0,
        }
    }
    
    fn identify_risks_and_opportunities(
        _graph: &HyperGraph,
        _node1: &HyperNode,
        _node2: &HyperNode,
        alignment_type: &AlignmentType,
    ) -> (Vec<String>, Vec<String>) {
        let mut risks = Vec::new();
        let mut opportunities = Vec::new();
        
        match alignment_type {
            AlignmentType::Perfect => {
                opportunities.push("High synergy potential".to_string());
                opportunities.push("Mutual benefit optimization".to_string());
                risks.push("Over-dependence risk".to_string());
            }
            AlignmentType::High => {
                opportunities.push("Strong collaboration potential".to_string());
                risks.push("Minor goal conflicts to resolve".to_string());
            }
            AlignmentType::Moderate => {
                opportunities.push("Partial collaboration possible".to_string());
                risks.push("Significant alignment work needed".to_string());
            }
            AlignmentType::Conflicting => {
                opportunities.push("Negotiation and compromise potential".to_string());
                risks.push("High coordination costs".to_string());
                risks.push("Potential for disputes".to_string());
            }
            AlignmentType::Incompatible => {
                risks.push("Fundamental incompatibility".to_string());
                risks.push("Likely negative outcomes".to_string());
            }
        }
        
        (risks, opportunities)
    }
    
    // Simplified centrality calculations
    fn calculate_betweenness_centrality(_graph: &HyperGraph, _node_id: Uuid) -> f64 {
        // TODO: Implement proper betweenness centrality calculation
        0.5 // Placeholder
    }
    
    fn calculate_closeness_centrality(_graph: &HyperGraph, _node_id: Uuid) -> f64 {
        // TODO: Implement proper closeness centrality calculation
        0.5 // Placeholder
    }
    
    fn calculate_eigenvector_centrality(_graph: &HyperGraph, _node_id: Uuid, neighbors: &[&HyperNode]) -> f64 {
        // Simple approximation based on neighbor count and their connections
        if neighbors.is_empty() {
            0.0
        } else {
            (neighbors.len() as f64).ln() / 10.0 // Rough approximation
        }
    }
}

#[derive(Debug, Clone)]
pub struct CentralityMeasures {
    pub degree: f64,
    pub betweenness: f64,
    pub closeness: f64,
    pub eigenvector: f64,
}