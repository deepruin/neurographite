use std::sync::Arc;
use tokio::net::TcpListener;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;

use crate::core::Database;

/// HTTP API server for Neurographite
pub struct Server {
    db: Arc<Database>,
}

#[derive(Debug, Deserialize)]
pub struct AddNodeRequest {
    pub data: serde_json::Value,
    pub node_type: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct AddNodeResponse {
    pub node_id: Uuid,
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct ConnectNodesRequest {
    pub node_ids: Vec<Uuid>,
    pub relationship: String,
    pub strength: f64,
}

#[derive(Debug, Serialize)]
pub struct ConnectNodesResponse {
    pub edge_id: Uuid,
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct FindSimilarRequest {
    pub node_id: Uuid,
    pub threshold: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct FindSimilarResponse {
    pub similar_nodes: Vec<SimilarNode>,
}

#[derive(Debug, Serialize)]
pub struct SimilarNode {
    pub node_id: Uuid,
    pub similarity_score: f64,
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub node_count: usize,
    pub edge_count: usize,
    pub total_spikes: u64,
    pub active_neurons: usize,
    pub average_activation: f64,
}

#[derive(Debug, Serialize)]
pub struct NetworkEffectResponse {
    pub affected_nodes: Vec<AffectedNode>,
    pub total_effect: f64,
    pub cascade_depth: usize,
}

#[derive(Debug, Serialize)]
pub struct AffectedNode {
    pub node_id: Uuid,
    pub effect_strength: f64,
}

impl Server {
    pub fn new(db: Database) -> Self {
        Self {
            db: Arc::new(db),
        }
    }
    
    pub async fn run(&self, addr: &str) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        tracing::info!("🌐 Neurographite API server listening on {}", addr);
        
        loop {
            let (stream, addr) = listener.accept().await?;
            let db = Arc::clone(&self.db);
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(db, stream).await {
                    tracing::error!("Error handling connection from {}: {}", addr, e);
                }
            });
        }
    }
    
    async fn handle_connection(
        db: Arc<Database>, 
        stream: tokio::net::TcpStream
    ) -> Result<()> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        
        let mut reader = BufReader::new(&stream);
        let mut line = String::new();
        
        // Read HTTP request line
        reader.read_line(&mut line).await?;
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        
        if parts.len() != 3 {
            return Self::send_error_response(stream, 400, "Bad Request").await;
        }
        
        let method = parts[0];
        let path = parts[1];
        
        // Skip headers for now (simple implementation)
        let mut headers = String::new();
        loop {
            headers.clear();
            reader.read_line(&mut headers).await?;
            if headers.trim().is_empty() {
                break;
            }
        }
        
        // Route the request
        match (method, path) {
            ("GET", "/health") => Self::handle_health(stream).await,
            ("GET", "/stats") => Self::handle_stats(db, stream).await,
            ("POST", "/nodes") => Self::handle_add_node(db, stream, reader).await,
            ("POST", "/edges") => Self::handle_connect_nodes(db, stream, reader).await,
            ("GET", path) if path.starts_with("/nodes/") && path.ends_with("/similar") => {
                Self::handle_find_similar(db, stream, path).await
            }
            ("GET", path) if path.starts_with("/nodes/") && path.ends_with("/network-effect") => {
                Self::handle_network_effect(db, stream, path).await
            }
            ("GET", "/relationships") => Self::handle_discover_relationships(db, stream).await,
            _ => Self::send_error_response(stream, 404, "Not Found").await,
        }
    }
    
    async fn handle_health(mut stream: tokio::net::TcpStream) -> Result<()> {
        let response = r#"{"status": "healthy", "service": "neurographite"}"#;
        Self::send_json_response(stream, 200, response).await
    }
    
    async fn handle_stats(db: Arc<Database>, stream: tokio::net::TcpStream) -> Result<()> {
        let stats = db.stats().await;
        let response = StatsResponse {
            node_count: stats.node_count,
            edge_count: stats.edge_count,
            total_spikes: stats.total_spikes,
            active_neurons: stats.active_neurons,
            average_activation: stats.average_activation,
        };
        
        let json = serde_json::to_string(&response)?;
        Self::send_json_response(stream, 200, &json).await
    }
    
    async fn handle_add_node(
        db: Arc<Database>,
        stream: tokio::net::TcpStream,
        mut reader: BufReader<&tokio::net::TcpStream>,
    ) -> Result<()> {
        // Read request body (simplified - assumes content-length is manageable)
        let mut body = String::new();
        reader.read_line(&mut body).await?;
        
        let request: AddNodeRequest = serde_json::from_str(&body)?;
        
        match db.add_node(request.data).await {
            Ok(node_id) => {
                let response = AddNodeResponse {
                    node_id,
                    success: true,
                };
                let json = serde_json::to_string(&response)?;
                Self::send_json_response(stream, 201, &json).await
            }
            Err(e) => {
                tracing::error!("Failed to add node: {}", e);
                Self::send_error_response(stream, 500, "Internal Server Error").await
            }
        }
    }
    
    async fn handle_connect_nodes(
        db: Arc<Database>,
        stream: tokio::net::TcpStream,
        mut reader: BufReader<&tokio::net::TcpStream>,
    ) -> Result<()> {
        let mut body = String::new();
        reader.read_line(&mut body).await?;
        
        let request: ConnectNodesRequest = serde_json::from_str(&body)?;
        
        match db.connect_nodes(request.node_ids, request.relationship, request.strength).await {
            Ok(edge_id) => {
                let response = ConnectNodesResponse {
                    edge_id,
                    success: true,
                };
                let json = serde_json::to_string(&response)?;
                Self::send_json_response(stream, 201, &json).await
            }
            Err(e) => {
                tracing::error!("Failed to connect nodes: {}", e);
                Self::send_error_response(stream, 500, "Internal Server Error").await
            }
        }
    }
    
    async fn handle_find_similar(
        db: Arc<Database>,
        stream: tokio::net::TcpStream,
        path: &str,
    ) -> Result<()> {
        // Extract node ID from path like "/nodes/{uuid}/similar"
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() < 3 {
            return Self::send_error_response(stream, 400, "Invalid path").await;
        }
        
        let node_id_str = parts[2];
        let node_id = Uuid::parse_str(node_id_str)
            .map_err(|_| anyhow::anyhow!("Invalid UUID"))?;
        
        match db.find_similar(node_id, 0.5).await {
            Ok(similar_nodes) => {
                let response = FindSimilarResponse {
                    similar_nodes: similar_nodes
                        .into_iter()
                        .map(|(id, score)| SimilarNode {
                            node_id: id,
                            similarity_score: score,
                        })
                        .collect(),
                };
                let json = serde_json::to_string(&response)?;
                Self::send_json_response(stream, 200, &json).await
            }
            Err(e) => {
                tracing::error!("Failed to find similar nodes: {}", e);
                Self::send_error_response(stream, 500, "Internal Server Error").await
            }
        }
    }
    
    async fn handle_network_effect(
        db: Arc<Database>,
        stream: tokio::net::TcpStream,
        path: &str,
    ) -> Result<()> {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() < 3 {
            return Self::send_error_response(stream, 400, "Invalid path").await;
        }
        
        let node_id_str = parts[2];
        let node_id = Uuid::parse_str(node_id_str)
            .map_err(|_| anyhow::anyhow!("Invalid UUID"))?;
        
        match db.simulate_network_effect(node_id, 1.0).await {
            Ok(effects) => {
                let response = NetworkEffectResponse {
                    affected_nodes: effects
                        .into_iter()
                        .map(|(id, strength)| AffectedNode {
                            node_id: id,
                            effect_strength: strength,
                        })
                        .collect(),
                    total_effect: 0.0, // TODO: Calculate total
                    cascade_depth: 0,  // TODO: Calculate depth
                };
                let json = serde_json::to_string(&response)?;
                Self::send_json_response(stream, 200, &json).await
            }
            Err(e) => {
                tracing::error!("Failed to simulate network effect: {}", e);
                Self::send_error_response(stream, 500, "Internal Server Error").await
            }
        }
    }
    
    async fn handle_discover_relationships(
        db: Arc<Database>,
        stream: tokio::net::TcpStream,
    ) -> Result<()> {
        match db.discover_relationships(10).await {
            Ok(relationships) => {
                let json = serde_json::to_string(&relationships)?;
                Self::send_json_response(stream, 200, &json).await
            }
            Err(e) => {
                tracing::error!("Failed to discover relationships: {}", e);
                Self::send_error_response(stream, 500, "Internal Server Error").await
            }
        }
    }
    
    async fn send_json_response(
        mut stream: tokio::net::TcpStream,
        status_code: u16,
        body: &str,
    ) -> Result<()> {
        let status_text = match status_code {
            200 => "OK",
            201 => "Created",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        };
        
        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            status_code, status_text, body.len(), body
        );
        
        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;
        Ok(())
    }
    
    async fn send_error_response(
        stream: tokio::net::TcpStream,
        status_code: u16,
        message: &str,
    ) -> Result<()> {
        let error_body = format!(r#"{{"error": "{}"}}"#, message);
        Self::send_json_response(stream, status_code, &error_body).await
    }
}