use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, BufReader};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;

use crate::core::Database;
use crate::frontend::StaticFileServer;

/// HTTP API server for Neurographite
pub struct Server {
    db: Arc<Database>,
    static_server: StaticFileServer,
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
            static_server: StaticFileServer::new("./frontend"),
        }
    }
    
    pub async fn run(&self, addr: &str) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        tracing::info!("🌐 Neurographite API server listening on {}", addr);
        
        loop {
            let (stream, addr) = listener.accept().await?;
            let db = Arc::clone(&self.db);
            let static_server = StaticFileServer::new("./frontend");
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(db, static_server, stream).await {
                    tracing::error!("Error handling connection from {}: {}", addr, e);
                }
            });
        }
    }
    
    async fn handle_connection(
        db: Arc<Database>,
        static_server: StaticFileServer,
        stream: tokio::net::TcpStream
    ) -> Result<()> {
        // Simple HTTP parsing - read first line only for now
        let mut buffer = vec![0; 1024];
        let n = stream.try_read(&mut buffer)?;
        let request = String::from_utf8_lossy(&buffer[..n]);
        let lines: Vec<&str> = request.lines().collect();
        
        if lines.is_empty() {
            return Self::send_error_response(stream, 400, "Bad Request").await;
        }
        
        let first_line = lines[0];
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() != 3 {
            return Self::send_error_response(stream, 400, "Bad Request").await;
        }
        
        let method = parts[0];
        let path = parts[1];
        
        // Route the request
        match (method, path) {
            // API routes
            ("GET", "/health") => Self::handle_health(stream).await,
            ("GET", "/stats") => Self::handle_stats(db, stream).await,
            ("POST", "/nodes") => Self::handle_add_node(db, stream, &request).await,
            ("POST", "/edges") => Self::handle_connect_nodes(db, stream, &request).await,
            ("GET", path) if path.starts_with("/nodes/") && path.ends_with("/similar") => {
                Self::handle_find_similar(db, stream, path).await
            }
            ("GET", path) if path.starts_with("/nodes/") && path.ends_with("/network-effect") => {
                Self::handle_network_effect(db, stream, path).await
            }
            ("GET", "/relationships") => Self::handle_discover_relationships(db, stream).await,
            
            // Handle CORS preflight
            ("OPTIONS", _) => Self::handle_cors_preflight(stream).await,
            
            // Static files (everything else)
            ("GET", path) => static_server.serve_file(path, stream).await,
            
            // Not found for non-GET requests
            _ => Self::send_error_response(stream, 404, "Not Found").await,
        }
    }
    
    async fn handle_health(stream: tokio::net::TcpStream) -> Result<()> {
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
        request: &str,
    ) -> Result<()> {
        // Extract JSON body from request (simplified)
        let body = if let Some(body_start) = request.find("\r\n\r\n") {
            &request[body_start + 4..]
        } else if let Some(body_start) = request.find("\n\n") {
            &request[body_start + 2..]
        } else {
            "{\"data\": {\"test\": true}}" // Default for testing
        };
        
        let request: AddNodeRequest = serde_json::from_str(body)?;
        
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
        request: &str,
    ) -> Result<()> {
        let body = if let Some(body_start) = request.find("\r\n\r\n") {
            &request[body_start + 4..]
        } else if let Some(body_start) = request.find("\n\n") {
            &request[body_start + 2..]
        } else {
            r#"{"node_ids": [], "relationship": "test", "strength": 0.5}"#
        };
        
        let request: ConnectNodesRequest = serde_json::from_str(body)?;
        
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
        stream: tokio::net::TcpStream,
        status_code: u16,
        body: &str,
    ) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        let mut stream = stream;
        let status_text = match status_code {
            200 => "OK",
            201 => "Created",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        };
        
        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Length: {}\r\n\r\n{}",
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
    
    async fn handle_cors_preflight(stream: tokio::net::TcpStream) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        let mut stream = stream;
        
        let response = "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Length: 0\r\n\r\n";
        
        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;
        Ok(())
    }
}